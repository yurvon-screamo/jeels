"""Upload the stable Windows installer to the CDN ``releases/`` prefix.

Microsoft Store submission requires a DIRECT installer link (HTTP 200, no
redirect chain); GitHub Releases URLs answer with a 302 to
``objects.githubusercontent.com`` and get the submission rejected. This
script uploads the fixed-name NSIS installer (the ADR-025 alias built by
``_build-tauri.yml``) to the S3-backed CDN behind ``s3.origa.uwuwu.net``,
producing two keys per stable release (ADR-041):

- ``releases/v<version>/Origa_x64-setup.exe`` — permanent versioned archive.
  Release-updated Cache-Control on purpose: a re-run of the same tag
  overwrites the key with different installer bytes, so ``immutable`` would
  poison the edge for a year (the PR #182 lesson).
- ``releases/latest/Origa_x64-setup.exe`` — the no-cache alias handed to
  Microsoft Store. Uploaded only AFTER the versioned key lands (pairing
  invariant: the alias must never point at a release whose archive failed).

Verification is layered once both uploads finish: an authenticated HEAD per
key (Cache-Control and Content-Length mandatory; the stored SHA256 only when
the store returns it in simple format — multipart uploads produce a
composite ``<base64>-<parts>`` checksum that never equals the plain file
hash), then a full public GET of both URLs comparing sha256 — the only check
that exercises the proxy edge on the real ~58 MB body.

Credentials come from the environment (CI) or the ``[origa]`` profile —
see ``_cdn_s3._s3_upload_client``.
"""

from __future__ import annotations

import argparse
import base64
import hashlib
import os
import re
import sys
import urllib.error
import urllib.request
from pathlib import Path
from typing import NoReturn

import _cdn_cache
import _cdn_s3

INSTALLER_NAME = "Origa_x64-setup.exe"
# A ~58 MB installer is 7 parts per key at 8 MB instead of ~3.5k sequential
# 16 KB PUTs; 8 MB parts are proven on T3 (see _cdn_s3._transfer_config).
UPLOAD_CHUNK_BYTES = 8 * 1024 * 1024
# Stable-only contract: rc/alpha/local builds must never reach the MS Store
# alias, so anything but a bare X.Y.Z is rejected outright.
STABLE_VERSION_RE = re.compile(r"^\d+\.\d+\.\d+$")
DEFAULT_CDN_BASE_URL = "https://s3.origa.uwuwu.net"
HASH_CHUNK_BYTES = 8192
GET_TIMEOUT_SECONDS = 300
USER_AGENT = "origa-upload-release/1.0"


def _fail(message: str) -> NoReturn:
    print(f"ERROR: {message}", file=sys.stderr)
    sys.exit(1)


def release_entries(version: str) -> list[tuple[str, str]]:
    """(key, Cache-Control) pairs; the versioned key MUST precede the alias."""
    versioned = f"releases/v{version}/{INSTALLER_NAME}"
    latest = f"releases/latest/{INSTALLER_NAME}"
    return [
        (versioned, _cdn_cache.cache_control_for(versioned)),
        (latest, _cdn_cache.cache_control_for(latest)),
    ]


def find_installer(artifact_dir: Path) -> Path:
    matches = sorted(artifact_dir.rglob(INSTALLER_NAME))
    if len(matches) != 1:
        _fail(
            f"expected exactly one {INSTALLER_NAME} under {artifact_dir}, "
            f"found {len(matches)}"
        )
    return matches[0]


def sha256_digests(path: Path) -> tuple[str, str]:
    """Return (hex, base64) SHA256 digests of the file."""
    hasher = hashlib.sha256()
    with path.open("rb") as f:
        while chunk := f.read(HASH_CHUNK_BYTES):
            hasher.update(chunk)
    digest = hasher.digest()
    return digest.hex(), base64.b64encode(digest).decode("ascii")


def verify_head(
    key: str,
    expected_cache_control: str,
    expected_size: int,
    expected_sha256_b64: str,
) -> None:
    # One checksum-enabled HEAD; if the store rejects ChecksumMode the
    # helper returns None and a plain HEAD retries — metadata checks stay
    # mandatory either way.
    metadata = _cdn_s3.stat_object(key, with_checksum=True)
    if metadata is None:
        metadata = _cdn_s3.stat_object(key)
    if metadata is None:
        _fail(f"HEAD failed for {key}")
    if metadata.cache_control != expected_cache_control:
        _fail(
            f"Cache-Control mismatch for {key}: "
            f"{metadata.cache_control!r} != {expected_cache_control!r}"
        )
    if metadata.content_length != expected_size:
        _fail(
            f"Content-Length mismatch for {key}: "
            f"{metadata.content_length} != {expected_size}"
        )

    checksum = metadata.checksum_sha256
    if checksum is None:
        print(
            f"  no stored checksum returned for {key}; "
            "integrity rests on the public GET check"
        )
    elif "-" in checksum:
        # Composite multipart checksum (checksum-of-checksums with a "-N"
        # part-count suffix); comparing it against the plain file hash is
        # never valid — the public GET check owns integrity here.
        print(f"  composite multipart checksum for {key}; integrity via GET")
    elif checksum != expected_sha256_b64:
        _fail(f"SHA256 mismatch for {key}: {checksum} != {expected_sha256_b64}")
    print(f"  HEAD ok: {key} [{metadata.cache_control}, {expected_size} B]")


def verify_public_get(url: str, expected_sha256_hex: str, expected_size: int) -> None:
    request = urllib.request.Request(url, headers={"User-Agent": USER_AGENT})
    hasher = hashlib.sha256()
    total = 0
    try:
        with urllib.request.urlopen(request, timeout=GET_TIMEOUT_SECONDS) as response:
            while chunk := response.read(HASH_CHUNK_BYTES):
                hasher.update(chunk)
                total += len(chunk)
    except (urllib.error.URLError, OSError, ValueError) as exc:
        # ValueError covers malformed URLs (e.g. an empty base URL collapsing
        # into a relative path) with an actionable message instead of a
        # traceback.
        _fail(f"public GET failed for {url}: {exc}")
    if total != expected_size or hasher.hexdigest() != expected_sha256_hex:
        _fail(
            f"public GET content mismatch for {url}: "
            f"{total} B (sha256 {hasher.hexdigest()[:16]}) "
            f"vs {expected_size} B expected"
        )
    print(f"  GET ok: {url} ({total} B, sha256 match)")


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Upload stable Windows installer to CDN releases/ (ADR-041)"
    )
    parser.add_argument(
        "--version",
        required=True,
        help="stable version, e.g. 1.2.3 (no v prefix, no rc suffix)",
    )
    parser.add_argument(
        "--artifact-dir",
        required=True,
        type=Path,
        help="directory containing Origa_x64-setup.exe (e.g. extracted "
        "windows-build artifact)",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="show planned uploads without touching S3",
    )
    args = parser.parse_args()

    if not STABLE_VERSION_RE.match(args.version):
        _fail(
            f"--version must be a stable semver (X.Y.Z), got {args.version!r}; "
            "rc/alpha builds must not reach the MS Store alias"
        )

    installer = find_installer(args.artifact_dir)
    size = installer.stat().st_size
    sha256_hex, sha256_b64 = sha256_digests(installer)
    entries = release_entries(args.version)
    # `or` (not a third get() default): an explicitly EMPTY variable must
    # fall back too — otherwise the GET URLs collapse into relative paths
    # and urlopen raises instead of verifying.
    base_url = (os.environ.get("ORIGA_CDN_BASE_URL") or DEFAULT_CDN_BASE_URL).rstrip("/")

    print(f"Installer: {installer} ({size} B)")
    print(f"Version: {args.version}")
    print(f"SHA256: {sha256_hex}")
    if args.dry_run:
        for key, cache_control in entries:
            print(f"  [DRY-RUN] {key} [{cache_control}]")
        return

    for key, cache_control in entries:
        print(f"Uploading {key} [{cache_control}]…")
        _cdn_s3.upload_file(
            installer,
            key,
            cache_control,
            dry_run=False,
            chunk_size=UPLOAD_CHUNK_BYTES,
            checksum_algorithm="SHA256",
        )

    print("\nVerifying (authenticated HEAD):")
    for key, cache_control in entries:
        verify_head(key, cache_control, size, sha256_b64)

    print("\nVerifying (public GET):")
    for key, _ in entries:
        verify_public_get(f"{base_url}/{key}", sha256_hex, size)

    print(f"\nMicrosoft Store direct link:\n  {base_url}/releases/latest/{INSTALLER_NAME}")


if __name__ == "__main__":
    main()
