"""S3 transport for ``deploy_cdn.py`` and ``refresh_cache_control.py``.

Two transports live here because T3 Storage breaks one of them:

- **aws CLI** (shelled out via ``pwsh`` on Windows — how the operator's
  PowerShell environment resolves the AWS wrapper): list-objects, head-object,
  copy-object (Cache-Control refresh), and manifest download. These are reads
  or server-side metadata copies; none uploads a request body, so T3's
  ~24KB single-PUT limit never applies.
- **boto3 multipart upload**: ``upload_file`` / ``sync_directory``. The aws
  CLI only auto-multiparts above its 8MB threshold, so files in the 24KB–8MB
  band (web fonts, audio, JSON) failed as a single PUT. boto3 with a 16KB
  ``TransferConfig`` threshold forces multipart and succeeds.

Centralising both keeps ``deploy_cdn.py`` and ``refresh_cache_control.py``
orchestrators rather than a transport layer.
"""

from __future__ import annotations

import json
import mimetypes
import os
import subprocess
import sys
import tempfile
from datetime import datetime
from pathlib import Path
from typing import TYPE_CHECKING, NamedTuple

if TYPE_CHECKING:
    from boto3.s3.transfer import TransferConfig
    from botocore.client import BaseClient

S3_BUCKET = "adaptable-foodbox-ucep7wx"
S3_PROFILE = "origa"
S3_ENDPOINT = "https://t3.storageapi.dev"

# copy-object caps at 5 GiB; surfaced so callers can skip oversize objects with
# a clear message instead of an opaque T3 error mid-walk.
COPY_OBJECT_MAX_BYTES = 5 * 1024 * 1024 * 1024

# The aws CLI is invoked through ``pwsh -Command`` (see run_aws_raw), which
# re-parses argv as a PowerShell script. A key containing a PowerShell
# metacharacter could therefore execute as a command/statement rather than a
# literal argument. This is a DENYLIST, not an ASCII allowlist: CJK and other
# Unicode letters are not command separators and pass through safely, which
# matters because kanji_animations/ uses the kanji themselves as filenames
# (一.svg, 丁.svg, ...).
_UNSAFE_KEY_CHARS = frozenset(" \t\r\n;|&`$\"'<>()@\\")


class ObjectMetadata(NamedTuple):
    cache_control: str | None
    content_length: int | None
    checksum_sha256: str | None = None


def s3_uri(key: str) -> str:
    return f"s3://{S3_BUCKET}/{key}"


def run_aws_raw(args: list[str]) -> subprocess.CompletedProcess[str]:
    cmd = ["pwsh", "-Command", "aws", *args]
    try:
        return subprocess.run(
            cmd,
            capture_output=True,
            text=True,
        )
    except FileNotFoundError:
        print("ERROR: 'aws' CLI not found.", file=sys.stderr)
        sys.exit(1)


def download_remote_manifest(dry_run: bool) -> dict[str, object] | None:
    # delete=False because the aws CLI runs as a separate process and re-opens
    # the path by name on Windows; cleanup happens in the ``finally`` below so
    # the temp file cannot leak even on dry-run / error paths.
    with tempfile.NamedTemporaryFile(suffix=".json", delete=False) as tmp:
        tmp_path = Path(tmp.name)

    try:
        if dry_run:
            print("  [DRY-RUN] would download remote manifest")
            return None

        result = run_aws_raw(
            [
                "s3",
                "cp",
                s3_uri("manifest.json"),
                str(tmp_path),
                "--profile",
                S3_PROFILE,
                "--endpoint-url",
                S3_ENDPOINT,
            ]
        )

        if result.returncode != 0:
            if "404" in result.stderr or "NoSuchKey" in result.stderr:
                print("  Remote manifest not found (first deployment)")
                return None
            print("ERROR: failed to download remote manifest", file=sys.stderr)
            print(result.stderr, file=sys.stderr)
            sys.exit(1)

        content = tmp_path.read_text(encoding="utf-8")
        return json.loads(content)
    finally:
        tmp_path.unlink(missing_ok=True)


def is_safe_key(key: str) -> bool:
    return bool(key) and not any(ch in _UNSAFE_KEY_CHARS for ch in key)


def filter_safe_keys(keys: list[str]) -> tuple[list[str], list[str]]:
    safe = [k for k in keys if is_safe_key(k)]
    unsafe = [k for k in keys if not is_safe_key(k)]
    return safe, unsafe


def _list_key_page(
    token: str | None, prefix: str | None = None
) -> tuple[list[str], str | None, bool]:
    """Fetch one list-objects-v2 page.

    Returns the page's keys, the continuation token for the next page (None if
    not truncated), and whether more pages remain.
    """
    args = [
        "s3api",
        "list-objects-v2",
        "--bucket",
        S3_BUCKET,
        "--profile",
        S3_PROFILE,
        "--endpoint-url",
        S3_ENDPOINT,
    ]
    if prefix:
        args += ["--prefix", prefix]
    if token:
        args += ["--continuation-token", token]
    result = run_aws_raw(args)
    if result.returncode != 0:
        print("ERROR: list-objects-v2 failed", file=sys.stderr)
        print(result.stderr, file=sys.stderr)
        sys.exit(1)

    data = json.loads(result.stdout) if result.stdout.strip() else {}
    keys = [
        obj["Key"]
        for obj in data.get("Contents", [])
        if isinstance(obj.get("Key"), str)
    ]
    return keys, data.get("NextContinuationToken"), bool(data.get("IsTruncated"))


def list_keys(prefix: str | None = None) -> tuple[list[str], int]:
    """List object keys (optionally under ``prefix``), paginating fully.

    Returns the safe keys plus a count of keys dropped as unsafe (shell
    metacharacters). Aborts if S3 signals truncation without a continuation
    token — that would silently lose keys.
    """
    raw: list[str] = []
    token: str | None = None
    while True:
        keys, next_token, truncated = _list_key_page(token, prefix)
        raw.extend(keys)
        if not truncated:
            break
        if not next_token:
            print(
                "ERROR: S3 returned IsTruncated without NextContinuationToken; "
                "cannot paginate safely",
                file=sys.stderr,
            )
            sys.exit(1)
        token = next_token

    safe, unsafe = filter_safe_keys(raw)
    for key in unsafe:
        print(f"  WARNING: dropping unsafe key: {key!r}", file=sys.stderr)
    return safe, len(unsafe)


def head_object(key: str) -> ObjectMetadata | None:
    result = run_aws_raw(
        [
            "s3api",
            "head-object",
            "--bucket",
            S3_BUCKET,
            "--key",
            key,
            "--profile",
            S3_PROFILE,
            "--endpoint-url",
            S3_ENDPOINT,
        ]
    )
    if result.returncode != 0:
        print(f"  WARNING: head-object failed for {key}", file=sys.stderr)
        print(result.stderr, file=sys.stderr)
        return None
    data = json.loads(result.stdout) if result.stdout.strip() else {}
    length = data.get("ContentLength")
    return ObjectMetadata(
        cache_control=data.get("CacheControl"),
        content_length=int(length) if isinstance(length, int) else None,
    )


def stat_object(key: str, *, with_checksum: bool = False) -> ObjectMetadata | None:
    """HEAD one object via boto3 — unlike :func:`head_object`, no pwsh/aws CLI
    wrapper, so this works on the Linux CI runner.

    ``with_checksum=True`` adds ``ChecksumMode=ENABLED`` so stores that keep
    upload checksums return them; a store that chokes on the mode makes the
    call fail and returns None, leaving the caller free to fall back to a
    plain stat. Returns None (with a printed warning) on any error, mirroring
    :func:`head_object` semantics.
    """
    from botocore.exceptions import BotoCoreError, ClientError

    params: dict[str, object] = {"Bucket": S3_BUCKET, "Key": key}
    if with_checksum:
        params["ChecksumMode"] = "ENABLED"
    try:
        data = _s3_upload_client().head_object(**params)
    except (BotoCoreError, ClientError) as exc:
        print(f"  WARNING: boto3 head-object failed for {key}: {exc}", file=sys.stderr)
        return None
    length = data.get("ContentLength")
    return ObjectMetadata(
        cache_control=data.get("CacheControl"),
        content_length=int(length) if isinstance(length, int) else None,
        checksum_sha256=data.get("ChecksumSHA256"),
    )


def copy_object_cache_control(key: str, target_cc: str, dry_run: bool) -> bool:
    """Rewrite one object's Cache-Control via a server-side self-copy.

    Returns True if applied (or previewed in dry-run), False if the copy
    failed — the caller continues with the remaining objects rather than
    aborting the whole walk.
    """
    args = [
        "s3api",
        "copy-object",
        "--bucket",
        S3_BUCKET,
        "--key",
        key,
        "--copy-source",
        f"{S3_BUCKET}/{key}",
        "--profile",
        S3_PROFILE,
        "--endpoint-url",
        S3_ENDPOINT,
        "--metadata-directive",
        "REPLACE",
        "--cache-control",
        target_cc,
    ]
    if dry_run:
        return True
    result = run_aws_raw(args)
    if result.returncode != 0:
        print(f"  ERROR: copy-object failed for {key}", file=sys.stderr)
        print(result.stderr, file=sys.stderr)
        return False
    return True


# Tigris (T3 Storage) single-PUT body limit ~24KB. Files above this must
# use multipart upload. 16KB threshold/chunk size verified to pass.
MULTIPART_THRESHOLD_BYTES = 16 * 1024

# Explicit pins for extensions whose canonical type matters and that mimetypes
# either cannot guess (woff/woff2) or resolves inconsistently across minimal
# installs (.json, .exe — the Windows installer must never be served as
# text/plain by a mis-guessing runner image).
_CONTENT_TYPE_OVERRIDES: dict[str, str] = {
    ".woff2": "font/woff2",
    ".woff": "font/woff",
    ".json": "application/json",
    ".exe": "application/octet-stream",
}


class RemoteObject(NamedTuple):
    size: int
    last_modified_epoch: float


_s3_client: BaseClient | None = None
_transfer_configs: dict[int, "TransferConfig"] = {}


def _s3_upload_client() -> BaseClient:
    """Return the shared boto3 S3 client used by every upload path.

    Credential-source contract: explicit environment credentials
    (``AWS_ACCESS_KEY_ID`` set) take precedence over the local ``[origa]``
    profile for ALL scripts built on this transport (``deploy_cdn.py``).
    CI exports scoped keys through the environment; operator machines keep
    the profile in ~/.aws/credentials and export nothing, so each side gets
    the credentials it owns. Exporting env credentials on an operator machine
    deliberately overrides the profile — useful for testing, but be aware the
    deploy then runs as that principal.

    boto3 stays imported lazily so refresh_cache_control.py — which only uses
    the aws-CLI helpers and never uploads — does not require boto3 to be
    installed just to import _cdn_s3.
    """
    global _s3_client
    if _s3_client is None:
        try:
            import boto3
        except ModuleNotFoundError:
            print(
                "ERROR: boto3 is not installed; run `uv sync` in scripts/.",
                file=sys.stderr,
            )
            sys.exit(1)
        from botocore.client import Config as BotoConfig
        if os.environ.get("AWS_ACCESS_KEY_ID"):
            session = boto3.Session()
        else:
            session = boto3.Session(profile_name=S3_PROFILE)
        _s3_client = session.client(
            "s3",
            endpoint_url=S3_ENDPOINT,
            config=BotoConfig(
                signature_version="s3v4",
                s3={"addressing_style": "virtual"},
                retries={"max_attempts": 5, "mode": "standard"},
            ),
        )
    return _s3_client


def _transfer_config(chunk_size: int = MULTIPART_THRESHOLD_BYTES) -> TransferConfig:
    """Multipart TransferConfig for ``chunk_size``, cached per size.

    The default 16KB threshold/chunk forces multipart for the small files
    T3's ~24KB single-PUT limit would reject. Large binaries (the ~58MB
    release installer) pass a bigger chunk so one key is ~7 parts instead
    of ~3.5k sequential PUTs — 8MB parts are proven on T3 (the aws CLI
    historically auto-multiparted above its own 8MB threshold). Configs are
    cached per chunk_size so repeated uploads reuse one object.
    """
    config = _transfer_configs.get(chunk_size)
    if config is None:
        from boto3.s3.transfer import TransferConfig

        config = TransferConfig(
            multipart_threshold=chunk_size,
            multipart_chunksize=chunk_size,
            max_concurrency=1,
        )
        _transfer_configs[chunk_size] = config
    return config


def content_type_for(path: Path) -> str:
    override = _CONTENT_TYPE_OVERRIDES.get(path.suffix.lower())
    if override:
        return override
    guessed, _ = mimetypes.guess_type(str(path))
    return guessed or "application/octet-stream"


def _upload_via_boto3(
    local_path: Path, key: str, extra_args: dict[str, str], chunk_size: int
) -> None:
    _s3_upload_client().upload_file(
        Filename=str(local_path),
        Bucket=S3_BUCKET,
        Key=key,
        ExtraArgs=extra_args,
        Config=_transfer_config(chunk_size),
    )


def upload_file(
    local_path: Path,
    key: str,
    cache_control: str,
    dry_run: bool,
    *,
    chunk_size: int = MULTIPART_THRESHOLD_BYTES,
    checksum_algorithm: str | None = None,
) -> None:
    """Upload one file to S3 via boto3 with a forced-low multipart threshold.

    A fresh PUT carries CacheControl/ContentType through ExtraArgs directly, so
    no separate metadata copy is needed. The 16KB threshold makes any body
    larger than that upload as multipart parts, sidestepping T3 Storage's
    single-PUT limit that breaks the aws CLI for 24KB-8MB files. boto3 errors
    abort the deploy with the offending key rather than a raw traceback.

    ``chunk_size`` overrides the multipart threshold/chunk (see
    :func:`_transfer_config`). ``checksum_algorithm`` (e.g. ``"SHA256"``)
    requests a stored checksum; S3-compatible stores that reject the checksum
    extension fail the upload — in that case it is retried once WITHOUT the
    checksum (a wide catch is deliberate: a non-checksum failure simply fails
    the retry identically) and integrity then rests on the caller's GET-side
    verification.
    """
    size = local_path.stat().st_size
    content_type = content_type_for(local_path)
    if dry_run:
        print(
            f"  [DRY-RUN] boto3 upload {local_path.name} -> {s3_uri(key)} "
            f"({size} B) [CacheControl={cache_control}, "
            f"ContentType={content_type}]"
        )
        return
    from boto3.exceptions import S3UploadFailedError as Boto3S3UploadFailedError
    from botocore.exceptions import BotoCoreError, ClientError
    from s3transfer.exceptions import RetriesExceededError, S3UploadFailedError

    # boto3 wraps transfer failures in its OWN S3UploadFailedError
    # (boto3.exceptions), distinct from s3transfer's class of the same name —
    # catching only the s3transfer flavor lets real upload failures escape
    # as raw tracebacks (seen live: UploadPart InternalError on T3).
    upload_errors = (
        BotoCoreError,
        ClientError,
        RetriesExceededError,
        S3UploadFailedError,
        Boto3S3UploadFailedError,
    )
    extra_args: dict[str, str] = {
        "CacheControl": cache_control,
        "ContentType": content_type,
    }
    if checksum_algorithm is not None:
        extra_args["ChecksumAlgorithm"] = checksum_algorithm
    try:
        _upload_via_boto3(local_path, key, extra_args, chunk_size)
    except upload_errors as exc:
        if checksum_algorithm is None:
            print(f"ERROR: boto3 upload failed for {key}: {exc}", file=sys.stderr)
            sys.exit(1)
        print(
            f"WARNING: checksum-enabled upload failed for {key} ({exc}); "
            "retrying without checksum",
            file=sys.stderr,
        )
        retry_args = {
            name: value
            for name, value in extra_args.items()
            if name != "ChecksumAlgorithm"
        }
        try:
            _upload_via_boto3(local_path, key, retry_args, chunk_size)
        except upload_errors as retry_exc:
            print(
                f"ERROR: boto3 upload failed for {key}: {retry_exc}",
                file=sys.stderr,
            )
            sys.exit(1)


def list_remote_objects(prefix: str) -> dict[str, RemoteObject]:
    """Map remote object keys under ``prefix`` to size + last-modified time.

    Paginates list-objects-v2 fully. ``sync_directory`` diffs this against
    local files (size + mtime) so unchanged static objects (100k+ kanji/audio/
    model files) are not re-uploaded on every deploy.
    """
    client = _s3_upload_client()
    normalized = prefix if prefix.endswith("/") else prefix + "/"
    objects: dict[str, RemoteObject] = {}
    paginator = client.get_paginator("list_objects_v2")
    for page in paginator.paginate(Bucket=S3_BUCKET, Prefix=normalized):
        for obj in page.get("Contents", []):
            obj_key = obj.get("Key")
            if not isinstance(obj_key, str):
                continue
            last_modified = obj.get("LastModified")
            last_modified_epoch = (
                last_modified.timestamp()
                if isinstance(last_modified, datetime)
                else 0.0
            )
            objects[obj_key] = RemoteObject(
                int(obj.get("Size", 0)), last_modified_epoch
            )
    return objects


def sync_directory(
    local_dir: Path, prefix: str, cache_control: str, dry_run: bool
) -> None:
    """Upload new/changed local files under ``local_dir`` to a bucket prefix.

    Mirrors ``aws s3 sync``: walk local files recursively, skip README.md, and
    upload only objects that are absent remotely, differ in byte size, or whose
    local mtime is newer than the remote LastModified. Shows a progress bar
    (count/total + percentage) for each directory.
    """
    if dry_run:
        return
    import time as _time

    base_prefix = prefix.rstrip("/") + "/"
    remote = list_remote_objects(prefix)

    # Count total files first for progress bar
    all_files = [
        p for p in sorted(local_dir.rglob("*"))
        if p.is_file() and p.name != "README.md"
    ]
    total = len(all_files)
    uploaded = 0
    skipped = 0
    start = _time.time()

    for i, local_path in enumerate(all_files, 1):
        key = base_prefix + local_path.relative_to(local_dir).as_posix()
        stat_result = local_path.stat()
        info = remote.get(key)
        if (
            info is not None
            and info.size == stat_result.st_size
            and stat_result.st_mtime <= info.last_modified_epoch
        ):
            skipped += 1
        else:
            upload_file(local_path, key, cache_control, dry_run)
            uploaded += 1

        if i % 200 == 0 or i == total:
            elapsed = _time.time() - start
            rate = i / elapsed if elapsed > 0 else 0
            pct = i * 100 // total
            print(
                f"    [{i}/{total}] {pct}%  up={uploaded} skip={skipped}  "
                f"{rate:.0f} files/s  {elapsed:.0f}s",
                flush=True,
            )
