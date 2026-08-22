"""Unit tests for ``upload_release_artifacts`` (ADR-041 MS Store direct link).

S3 and HTTP are external systems — every transport call (boto3 upload, boto3
HEAD, public GET) is monkeypatched. The decision logic under test: key
scheme + ordering (versioned before the latest alias — the pairing
invariant), stable-only version validation, the layered verification
verdicts (Cache-Control, size, simple-vs-composite checksum, GET hash), and
the happy-path orchestration.
"""

from __future__ import annotations

import base64
import hashlib
import sys
from pathlib import Path

import pytest

import _cdn_s3
import upload_release_artifacts as ura


class _FakeStat:
    """Serves canned ``stat_object`` results per key."""

    def __init__(self, metadata: dict[str, _cdn_s3.ObjectMetadata | None]) -> None:
        self._metadata = metadata
        self.calls: list[tuple[str, bool]] = []

    def __call__(
        self, key: str, *, with_checksum: bool = False
    ) -> _cdn_s3.ObjectMetadata | None:
        self.calls.append((key, with_checksum))
        return self._metadata.get(key)


class _FakeResponse:
    """Minimal ``urlopen`` context manager yielding a byte payload."""

    def __init__(self, payload: bytes) -> None:
        self._payload = payload

    def read(self, size: int = -1) -> bytes:
        if not self._payload:
            return b""
        chunk, self._payload = self._payload[:size], self._payload[size:]
        return chunk

    def __enter__(self) -> "_FakeResponse":
        return self

    def __exit__(self, *_args: object) -> None:
        return None


def _make_installer(tmp_path: Path, payload: bytes = b"installer-bytes") -> Path:
    nested = tmp_path / "bundle" / "nsis"
    nested.mkdir(parents=True)
    installer = nested / ura.INSTALLER_NAME
    installer.write_bytes(payload)
    return installer


def _digests(payload: bytes) -> tuple[str, str]:
    digest = hashlib.sha256(payload).digest()
    return digest.hex(), base64.b64encode(digest).decode("ascii")


# ---------------------------------------------------------------------------
# Key scheme and ordering — the pairing invariant
# ---------------------------------------------------------------------------


def test_release_entries_versioned_key_precedes_latest_alias():
    entries = ura.release_entries("1.2.3")

    assert [key for key, _ in entries] == [
        "releases/1.2.3/Origa_x64-setup.exe",
        "releases/latest/Origa_x64-setup.exe",
    ]
    # Policies come from the shared tiered cache: archive is revalidatable
    # (tag re-runs overwrite with different bytes), alias is always-fresh.
    assert entries[0][1] == "public, max-age=300, must-revalidate"
    assert entries[1][1] == "no-cache"


# ---------------------------------------------------------------------------
# find_installer
# ---------------------------------------------------------------------------


def test_find_installer_returns_the_single_match(tmp_path):
    installer = _make_installer(tmp_path)

    assert ura.find_installer(tmp_path) == installer


def test_find_installer_rejects_missing_and_ambiguous(tmp_path):
    with pytest.raises(SystemExit) as missing:
        ura.find_installer(tmp_path)
    assert missing.value.code == 1

    _make_installer(tmp_path)
    _make_installer(tmp_path / "second")

    with pytest.raises(SystemExit) as ambiguous:
        ura.find_installer(tmp_path)
    assert ambiguous.value.code == 1


# ---------------------------------------------------------------------------
# Version validation — stable-only contract
# ---------------------------------------------------------------------------


@pytest.mark.parametrize(
    "version",
    ["1.2.3-rc1", "1.2.3-rc.4", "v1.2.3", "1.2", "1.2.3.4", "abc", ""],
)
def test_non_stable_version_is_rejected(version, tmp_path, monkeypatch):
    _make_installer(tmp_path)
    monkeypatch.setattr(
        sys, "argv", ["prog", "--version", version, "--artifact-dir", str(tmp_path)]
    )

    with pytest.raises(SystemExit) as exc:
        ura.main()

    assert exc.value.code == 1


def test_stable_version_passes_validation(tmp_path, monkeypatch):
    _make_installer(tmp_path)
    monkeypatch.setattr(
        sys,
        "argv",
        ["prog", "--version", "1.2.3", "--artifact-dir", str(tmp_path), "--dry-run"],
    )

    ura.main()  # does not raise


# ---------------------------------------------------------------------------
# Dry run — must touch nothing remote
# ---------------------------------------------------------------------------


def test_dry_run_prints_plan_without_uploads(tmp_path, monkeypatch, capsys):
    _make_installer(tmp_path)

    def fail_upload(*_args: object, **_kwargs: object) -> None:
        raise AssertionError("dry-run must not upload")

    monkeypatch.setattr(_cdn_s3, "upload_file", fail_upload)
    monkeypatch.setattr(
        sys,
        "argv",
        ["prog", "--version", "1.2.3", "--artifact-dir", str(tmp_path), "--dry-run"],
    )

    ura.main()

    out = capsys.readouterr().out
    assert "[DRY-RUN] releases/1.2.3/Origa_x64-setup.exe" in out
    assert "[DRY-RUN] releases/latest/Origa_x64-setup.exe" in out


# ---------------------------------------------------------------------------
# Upload orchestration — order, chunk, checksum
# ---------------------------------------------------------------------------


def test_main_uploads_both_keys_versioned_first(tmp_path, monkeypatch):
    _make_installer(tmp_path)
    uploads: list[dict[str, object]] = []

    def fake_upload(_path, key, _cc, *, dry_run, **kwargs):
        uploads.append({"key": key, "dry": dry_run, **kwargs})

    def fake_verify_head(key, cache_control, size, sha_b64):
        assert cache_control in ("public, max-age=300, must-revalidate", "no-cache")

    def fake_verify_get(url, sha_hex, size):
        assert url.startswith("https://")

    monkeypatch.setattr(_cdn_s3, "upload_file", fake_upload)
    monkeypatch.setattr(ura, "verify_head", fake_verify_head)
    monkeypatch.setattr(ura, "verify_public_get", fake_verify_get)
    monkeypatch.setattr(
        sys,
        "argv",
        ["prog", "--version", "1.2.3", "--artifact-dir", str(tmp_path)],
    )

    ura.main()

    assert [u["key"] for u in uploads] == [
        "releases/1.2.3/Origa_x64-setup.exe",
        "releases/latest/Origa_x64-setup.exe",
    ]
    assert all(u["dry"] is False for u in uploads)
    assert all(u["chunk_size"] == ura.UPLOAD_CHUNK_BYTES for u in uploads)
    assert all(u["checksum_algorithm"] == "SHA256" for u in uploads)


# ---------------------------------------------------------------------------
# verify_head — layered verdicts
# ---------------------------------------------------------------------------


def _ok_metadata(payload: bytes, checksum: str | None) -> _cdn_s3.ObjectMetadata:
    return _cdn_s3.ObjectMetadata(
        cache_control="public, max-age=300, must-revalidate",
        content_length=len(payload),
        checksum_sha256=checksum,
    )


def test_verify_head_accepts_simple_checksum_match(monkeypatch, capsys):
    payload = b"x" * 100
    _, sha_b64 = _digests(payload)
    key = "releases/1.2.3/Origa_x64-setup.exe"
    monkeypatch.setattr(
        _cdn_s3,
        "stat_object",
        _FakeStat({key: _ok_metadata(payload, sha_b64)}),
    )

    ura.verify_head(key, "public, max-age=300, must-revalidate", len(payload), sha_b64)

    out = capsys.readouterr().out
    # The simple-format match passes silently through to the final line.
    assert "HEAD ok" in out
    assert "mismatch" not in out


def test_verify_head_treats_composite_checksum_as_informational(
    monkeypatch, capsys
):
    # Multipart uploads store a composite "<base64>-<parts>" checksum that
    # never equals the plain file hash; treating it as a mismatch would fail
    # every stable release. Integrity rests on the public GET check.
    payload = b"x" * 100
    _, sha_b64 = _digests(payload)
    key = "releases/1.2.3/Origa_x64-setup.exe"
    composite = "nQJRx6+aaaaaaaaaaaaa==-7"
    monkeypatch.setattr(
        _cdn_s3, "stat_object", _FakeStat({key: _ok_metadata(payload, composite)})
    )

    ura.verify_head(key, "public, max-age=300, must-revalidate", len(payload), sha_b64)

    out = capsys.readouterr().out
    assert "composite" in out
    assert "SHA256 ok" not in out


def test_verify_head_fails_on_simple_checksum_mismatch(monkeypatch):
    payload = b"x" * 100
    _, local_b64 = _digests(payload)
    key = "releases/1.2.3/Origa_x64-setup.exe"
    wrong_b64 = base64.b64encode(hashlib.sha256(b"other").digest()).decode("ascii")
    monkeypatch.setattr(
        _cdn_s3, "stat_object", _FakeStat({key: _ok_metadata(payload, wrong_b64)})
    )

    with pytest.raises(SystemExit) as exc:
        ura.verify_head(key, "public, max-age=300, must-revalidate", len(payload), local_b64)

    assert exc.value.code == 1


def test_verify_head_fails_on_cache_control_mismatch(monkeypatch):
    # Only Cache-Control differs (size matches): a policy regression must
    # fail verification even when the object itself is intact.
    payload = b"x" * 10
    key = "releases/latest/Origa_x64-setup.exe"
    _, local_b64 = _digests(payload)
    monkeypatch.setattr(
        _cdn_s3, "stat_object", _FakeStat({key: _ok_metadata(payload, None)})
    )

    with pytest.raises(SystemExit) as exc:
        ura.verify_head(key, "no-cache", len(payload), local_b64)

    assert exc.value.code == 1


# ---------------------------------------------------------------------------
# verify_public_get — the only end-to-end integrity check
# ---------------------------------------------------------------------------


def test_verify_public_get_accepts_matching_body(monkeypatch):
    payload = b"y" * 5000
    sha_hex, _ = _digests(payload)
    monkeypatch.setattr(
        ura.urllib.request, "urlopen", lambda *_a, **_k: _FakeResponse(payload)
    )

    ura.verify_public_get("https://cdn.example/releases/latest/x.exe", sha_hex, len(payload))


def test_verify_public_get_fails_on_corrupted_body(monkeypatch):
    payload = b"y" * 5000
    sha_hex, _ = _digests(payload)
    monkeypatch.setattr(
        ura.urllib.request, "urlopen", lambda *_a, **_k: _FakeResponse(b"corrupt")
    )

    with pytest.raises(SystemExit) as exc:
        ura.verify_public_get("https://cdn.example/x.exe", sha_hex, len(payload))

    assert exc.value.code == 1


def test_verify_public_get_fails_on_network_error(monkeypatch):
    import urllib.error

    def raise_urlerror(*_a: object, **_k: object):
        raise urllib.error.URLError("proxy timeout")

    monkeypatch.setattr(ura.urllib.request, "urlopen", raise_urlerror)

    with pytest.raises(SystemExit) as exc:
        ura.verify_public_get("https://cdn.example/x.exe", "0" * 64, 10)

    assert exc.value.code == 1


# ---------------------------------------------------------------------------
# Happy path end to end (all transports faked)
# ---------------------------------------------------------------------------


def test_verify_head_falls_back_to_plain_head_when_checksum_mode_rejected(
    monkeypatch, capsys
):
    # A store that rejects ChecksumMode makes the first HEAD return None;
    # the plain retry must keep the mandatory metadata checks alive.
    payload = b"x" * 10
    key = "releases/1.2.3/Origa_x64-setup.exe"
    _, local_b64 = _digests(payload)
    plain = _cdn_s3.ObjectMetadata(
        cache_control="public, max-age=300, must-revalidate",
        content_length=len(payload),
        checksum_sha256=None,
    )
    calls: list[bool] = []

    def stat(key_called, *, with_checksum=False):
        calls.append(with_checksum)
        return None if with_checksum else plain

    monkeypatch.setattr(_cdn_s3, "stat_object", stat)

    ura.verify_head(
        key, "public, max-age=300, must-revalidate", len(payload), local_b64
    )

    assert calls == [True, False]
    out = capsys.readouterr().out
    assert "HEAD ok" in out
    assert "no stored checksum" in out


def test_empty_cdn_base_url_env_falls_back_to_default(tmp_path, monkeypatch, capsys):
    # An explicitly empty ORIGA_CDN_BASE_URL must not collapse GET URLs
    # into relative paths — `or` falls back to the production default.
    payload = b"installer"
    _make_installer(tmp_path, payload)
    sha_hex, sha_b64 = _digests(payload)
    stat_map = {
        "releases/1.2.3/Origa_x64-setup.exe": _cdn_s3.ObjectMetadata(
            "public, max-age=300, must-revalidate", len(payload), None
        ),
        "releases/latest/Origa_x64-setup.exe": _cdn_s3.ObjectMetadata(
            "no-cache", len(payload), None
        ),
    }
    monkeypatch.setattr(_cdn_s3, "upload_file", lambda *a, **k: None)
    monkeypatch.setattr(_cdn_s3, "stat_object", _FakeStat(stat_map))
    monkeypatch.setattr(
        ura.urllib.request, "urlopen", lambda *_a, **_k: _FakeResponse(payload)
    )
    monkeypatch.setattr(sys, "argv", ["prog", "--version", "1.2.3", "--artifact-dir", str(tmp_path)])
    monkeypatch.setenv("ORIGA_CDN_BASE_URL", "")

    ura.main()

    out = capsys.readouterr().out
    assert ura.DEFAULT_CDN_BASE_URL in out


def test_main_happy_path_prints_ms_store_link(tmp_path, monkeypatch, capsys):
    payload = b"installer"
    _make_installer(tmp_path, payload)
    sha_hex, sha_b64 = _digests(payload)
    stat_map = {
        "releases/1.2.3/Origa_x64-setup.exe": _cdn_s3.ObjectMetadata(
            "public, max-age=300, must-revalidate", len(payload), sha_b64
        ),
        "releases/latest/Origa_x64-setup.exe": _cdn_s3.ObjectMetadata(
            "no-cache", len(payload), None
        ),
    }
    uploads: list[str] = []

    def fake_upload(_path, key, _cc, *, dry_run, **_kwargs):
        assert dry_run is False
        uploads.append(key)

    monkeypatch.setattr(_cdn_s3, "upload_file", fake_upload)
    monkeypatch.setattr(_cdn_s3, "stat_object", _FakeStat(stat_map))
    monkeypatch.setattr(
        ura.urllib.request, "urlopen", lambda *_a, **_k: _FakeResponse(payload)
    )
    monkeypatch.setattr(
        sys,
        "argv",
        ["prog", "--version", "1.2.3", "--artifact-dir", str(tmp_path)],
    )
    monkeypatch.delenv("ORIGA_CDN_BASE_URL", raising=False)

    ura.main()

    out = capsys.readouterr().out
    assert uploads == [
        "releases/1.2.3/Origa_x64-setup.exe",
        "releases/latest/Origa_x64-setup.exe",
    ]
    assert (
        f"{ura.DEFAULT_CDN_BASE_URL}/releases/latest/{ura.INSTALLER_NAME}" in out
    )
    # The store submission contract: the VERSIONED URL (no "v" prefix) is
    # what goes into the MS Store form, and it must be labeled as such.
    assert (
        f"{ura.DEFAULT_CDN_BASE_URL}/releases/1.2.3/{ura.INSTALLER_NAME}" in out
    )
    assert "store submission (versioned)" in out
    # Bootstrap contract: the full sha256 (for cross-checking against the
    # GitHub Release digest) and the version are part of the output.
    sha_hex, _ = _digests(payload)
    assert f"SHA256: {sha_hex}" in out
    assert "Version: 1.2.3" in out
    assert "GET ok" in out
