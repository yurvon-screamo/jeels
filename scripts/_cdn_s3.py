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
import subprocess
import sys
import tempfile
from datetime import datetime
from pathlib import Path
from typing import TYPE_CHECKING, NamedTuple

if TYPE_CHECKING:
    from boto3.s3.transfer import TransferConfig
    from botocore.client import BaseClient

S3_BUCKET = "origa-cdn"
S3_PROFILE = "origa-cdn"
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


# T3 Storage drops single-PUT request bodies larger than ~24KB. The aws CLI
# only switches to multipart above its 8MB default threshold, so files in the
# 24KB-8MB band (fonts, audio, JSON) were sent as one PUT and failed. Both the
# multipart threshold and the part size are capped by T3's ~24KB per-PUT body
# limit, so they share one 16KB value -- the largest size verified to pass
# (all 8 web fonts deployed through it).
#
# Trade-off of the tiny part size: large objects become many parts. The biggest
# objects here are the whisper decoder (118 MB -> ~7200 parts) and the ndlocr
# and whisper-encoder models (33-41 MB). S3 caps one object at 10 000 parts,
# so at 16 KB the per-object ceiling is ~156 MB; the current largest object
# sits under it, but a future model beyond that needs a larger part size, which
# in turn requires pinning T3's exact PUT limit (only "~24 KB" is known today).
# A model swap is also slow (thousands of serial parts); steady-state deploys
# are unaffected because sync_directory skips unchanged objects by size+mtime.
# Set a bucket lifecycle rule to abort incomplete multipart uploads so a failed
# large upload does not accumulate storage cost.
MULTIPART_THRESHOLD_BYTES = 16 * 1024

# Explicit pins for extensions whose canonical type matters and that mimetypes
# either cannot guess (woff/woff2) or resolves inconsistently across minimal
# installs (.json).
_CONTENT_TYPE_OVERRIDES: dict[str, str] = {
    ".woff2": "font/woff2",
    ".woff": "font/woff",
    ".json": "application/json",
}


class RemoteObject(NamedTuple):
    size: int
    last_modified_epoch: float


_s3_client: BaseClient | None = None
_transfer_config_obj: TransferConfig | None = None


def _s3_upload_client() -> BaseClient:
    # boto3 is imported lazily so refresh_cache_control.py -- which only uses
    # the aws-CLI helpers above and never uploads -- does not require boto3 to
    # be installed just to import _cdn_s3.
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
        session = boto3.Session(profile_name=S3_PROFILE)
        _s3_client = session.client("s3", endpoint_url=S3_ENDPOINT)
    return _s3_client


def _transfer_config() -> TransferConfig:
    global _transfer_config_obj
    if _transfer_config_obj is None:
        from boto3.s3.transfer import TransferConfig

        _transfer_config_obj = TransferConfig(
            multipart_threshold=MULTIPART_THRESHOLD_BYTES,
            multipart_chunksize=MULTIPART_THRESHOLD_BYTES,
            max_concurrency=1,
        )
    return _transfer_config_obj


def content_type_for(path: Path) -> str:
    override = _CONTENT_TYPE_OVERRIDES.get(path.suffix.lower())
    if override:
        return override
    guessed, _ = mimetypes.guess_type(str(path))
    return guessed or "application/octet-stream"


def upload_file(local_path: Path, key: str, cache_control: str, dry_run: bool) -> None:
    """Upload one file to S3 via boto3 with a forced-low multipart threshold.

    A fresh PUT carries CacheControl/ContentType through ExtraArgs directly, so
    no separate metadata copy is needed. The 16KB threshold makes any body
    larger than that upload as multipart parts, sidestepping T3 Storage's
    single-PUT limit that breaks the aws CLI for 24KB-8MB files. boto3 errors
    abort the deploy with the offending key rather than a raw traceback.
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
    from botocore.exceptions import BotoCoreError, ClientError
    from s3transfer.exceptions import RetriesExceededError, S3UploadFailedError

    try:
        _s3_upload_client().upload_file(
            Filename=str(local_path),
            Bucket=S3_BUCKET,
            Key=key,
            ExtraArgs={"CacheControl": cache_control, "ContentType": content_type},
            Config=_transfer_config(),
        )
    except (
        BotoCoreError,
        ClientError,
        RetriesExceededError,
        S3UploadFailedError,
    ) as exc:
        print(f"ERROR: boto3 upload failed for {key}: {exc}", file=sys.stderr)
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
    local mtime is newer than the remote LastModified (the CLI's size+mtime
    heuristic, so a same-size content edit is still re-uploaded rather than
    silently served stale). The deploy orchestrator prints a per-directory
    header before calling this, so in dry-run the function does nothing -- it
    neither walks the 100k+ local tree nor lists remote, keeping the preview
    instant and offline. Each upload routes through ``upload_file``, so the
    16KB multipart threshold applies.
    """
    if dry_run:
        return
    base_prefix = prefix.rstrip("/") + "/"
    remote = list_remote_objects(prefix)
    for local_path in sorted(local_dir.rglob("*")):
        if not local_path.is_file():
            continue
        if local_path.name == "README.md":
            continue
        key = base_prefix + local_path.relative_to(local_dir).as_posix()
        stat_result = local_path.stat()
        info = remote.get(key)
        if (
            info is not None
            and info.size == stat_result.st_size
            and stat_result.st_mtime <= info.last_modified_epoch
        ):
            continue
        upload_file(local_path, key, cache_control, dry_run)
