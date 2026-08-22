"""Unit tests for the boto3 upload path in ``_cdn_s3`` (T3 Storage fix).

T3 Storage drops single-PUT bodies larger than ~24KB; the aws CLI only
auto-multiparts above 8MB, so the 24KB-8MB band (fonts, audio, JSON) failed.
These tests pin the behaviour added to replace it: the multipart threshold,
the content-type resolver, the per-file upload metadata + error handling, and
the directory sync diff (size + mtime) that avoids re-uploading unchanged
static objects.

boto3 is an external system, so the S3 client and the network-touching
helpers are monkeypatched. The decision logic (which files to upload) is
exercised black-box against a fake remote object map.
"""

from __future__ import annotations

from datetime import datetime, timezone
from pathlib import Path

import pytest

import _cdn_s3
from _cdn_s3 import (
    MULTIPART_THRESHOLD_BYTES,
    RemoteObject,
    _transfer_config,
    content_type_for,
    list_remote_objects,
    sync_directory,
    upload_file,
)


class _FakeUploadClient:
    """Records boto3 ``upload_file`` calls without touching the network."""

    def __init__(self, raise_exc: BaseException | None = None) -> None:
        self.calls: list[dict[str, object]] = []
        self._raise_exc = raise_exc

    def upload_file(self, **kwargs: object) -> None:
        if self._raise_exc is not None:
            raise self._raise_exc
        self.calls.append(kwargs)


class _FakePaginator:
    def __init__(self, pages: list[dict[str, object]]) -> None:
        self._pages = pages

    def paginate(self, **_kwargs: object) -> object:
        return iter(self._pages)


class _FakeListClient:
    """Yields canned list-objects-v2 pages for ``list_remote_objects``."""

    def __init__(self, pages: list[dict[str, object]]) -> None:
        self._pages = pages

    def get_paginator(self, _name: str) -> _FakePaginator:
        return _FakePaginator(self._pages)


def _record_uploads() -> tuple[list[tuple[Path, str, str, bool]], object]:
    """Return (calls, fake_upload_file) capturing each forwarded upload."""
    calls: list[tuple[Path, str, str, bool]] = []

    def fake(local_path: Path, key: str, cache_control: str, dry_run: bool) -> None:
        calls.append((local_path, key, cache_control, dry_run))

    return calls, fake


def _build_sync_dir(root: Path) -> Path:
    """A sample dir: two ASCII files, a README, a nested file, and a CJK file."""
    d = root / "assets"
    d.mkdir()
    (d / "a.woff2").write_bytes(b"AAAA")  # 4 bytes
    (d / "b.json").write_bytes(b"BBBBBBBB")  # 8 bytes
    (d / "README.md").write_text("readme")
    nested = d / "sub"
    nested.mkdir()
    (nested / "c.bin").write_bytes(b"CC")  # 2 bytes
    # CJK filename mirrors the real kanji_animations naming (一.svg); the key
    # must round-trip without shell metacharacter concerns (boto3 path).
    (d / "一.svg").write_bytes(b"DDDD")
    return d


# ---------------------------------------------------------------------------
# TransferConfig threshold — the actual fix parameter
# ---------------------------------------------------------------------------


def test_multipart_threshold_forces_multipart_under_cli_default():
    # The aws CLI's auto-multipart kicks in at 8MB; we must force it down to
    # 16KB so T3's ~24KB single-PUT limit never applies. A 2MB font (the
    # failing case) is now well above the threshold -> multipart.
    assert MULTIPART_THRESHOLD_BYTES == 16 * 1024
    cfg = _transfer_config()
    assert cfg.multipart_threshold == 16 * 1024
    assert cfg.multipart_chunksize == 16 * 1024
    assert 2 * 1024 * 1024 > MULTIPART_THRESHOLD_BYTES


def test_transfer_config_is_cached_per_chunk_size():
    # ADR-041: the ~58MB installer uploads with 8MB parts; the cache must be
    # keyed by chunk_size or a large-file upload would silently reuse the
    # 16KB config (3.5k sequential PUTs instead of 7).
    _cdn_s3._transfer_configs.clear()
    cfg_16k = _transfer_config(16 * 1024)
    cfg_8m = _transfer_config(8 * 1024 * 1024)

    assert cfg_16k is not cfg_8m
    assert cfg_8m.multipart_threshold == 8 * 1024 * 1024
    assert cfg_8m.multipart_chunksize == 8 * 1024 * 1024
    # Repeated requests reuse the cached object.
    assert _transfer_config(8 * 1024 * 1024) is cfg_8m
    assert _transfer_config() is cfg_16k


# ---------------------------------------------------------------------------
# content_type_for
# ---------------------------------------------------------------------------


@pytest.mark.parametrize(
    "filename,expected",
    [
        ("noto-sans-jp-400.woff2", "font/woff2"),
        ("cormorant.woff", "font/woff"),
        ("grammar.json", "application/json"),
        # Override lookup is case-insensitive — real extensions vary in case.
        ("UPPER.WOFF2", "font/woff2"),
        # The Windows installer must never depend on a runner image's
        # mimetypes registry (ADR-041).
        ("Origa_x64-setup.exe", "application/octet-stream"),
    ],
)
def test_content_type_override(filename: str, expected: str):
    assert content_type_for(Path(filename)) == expected


def test_content_type_unknown_falls_back_to_octet_stream():
    # mimetypes cannot guess .onnx -> default binary type.
    assert content_type_for(Path("model.onnx")) == "application/octet-stream"


def test_content_type_cjk_filename_resolves():
    # Kanji SVGs use the kanji as the filename (一.svg); suffix lookup must
    # not be confused by the CJK base name.
    assert content_type_for(Path("一.svg")) == "image/svg+xml"


# ---------------------------------------------------------------------------
# upload_file
# ---------------------------------------------------------------------------


def test_upload_file_passes_metadata_and_transfer_config(tmp_path, monkeypatch):
    local = tmp_path / "noto.woff2"
    local.write_bytes(b"x" * 100)
    fake = _FakeUploadClient()
    monkeypatch.setattr(_cdn_s3, "_s3_upload_client", lambda: fake)

    upload_file(
        local, "fonts/noto.woff2", "public, max-age=1, immutable", dry_run=False
    )

    assert len(fake.calls) == 1
    call = fake.calls[0]
    assert call["Bucket"] == _cdn_s3.S3_BUCKET
    assert call["Key"] == "fonts/noto.woff2"
    assert call["Filename"] == str(local)
    assert call["ExtraArgs"] == {
        "CacheControl": "public, max-age=1, immutable",
        "ContentType": "font/woff2",
    }


def test_upload_file_dry_run_skips_client(tmp_path, monkeypatch, capsys):
    local = tmp_path / "index.json"
    local.write_text("{}")
    fake = _FakeUploadClient()
    monkeypatch.setattr(_cdn_s3, "_s3_upload_client", lambda: fake)

    upload_file(local, "grammar/grammar.json", "no-cache", dry_run=True)

    assert fake.calls == []
    out = capsys.readouterr().out
    assert "[DRY-RUN]" in out
    assert "grammar/grammar.json" in out
    assert "application/json" in out
    assert "no-cache" in out


def test_upload_file_aborts_with_key_on_boto3_error(tmp_path, monkeypatch):
    # A failing upload must surface the offending key and exit non-zero, not
    # bubble a raw botocore traceback.
    from botocore.exceptions import ClientError

    local = tmp_path / "broken.woff2"
    local.write_bytes(b"x" * 10)
    err = ClientError({"Error": {"Code": "SlowDown", "Message": "boom"}}, "PutObject")
    monkeypatch.setattr(_cdn_s3, "_s3_upload_client", lambda: _FakeUploadClient(err))

    with pytest.raises(SystemExit) as exc:
        upload_file(local, "fonts/broken.woff2", "immutable", dry_run=False)

    assert exc.value.code == 1


def test_upload_file_aborts_on_s3transfer_retry_error(tmp_path, monkeypatch):
    # s3transfer raises its own exceptions (not BotoCoreError) on retry
    # exhaustion / part failure -- realistic on a flaky T3 endpoint -- and
    # they must surface the offending key, not a raw traceback.
    from s3transfer.exceptions import RetriesExceededError

    local = tmp_path / "flaky.woff2"
    local.write_bytes(b"x" * 10)
    err = RetriesExceededError(last_exception=RuntimeError("timeout"))
    monkeypatch.setattr(_cdn_s3, "_s3_upload_client", lambda: _FakeUploadClient(err))

    with pytest.raises(SystemExit) as exc:
        upload_file(local, "fonts/flaky.woff2", "immutable", dry_run=False)

    assert exc.value.code == 1


# ---------------------------------------------------------------------------
# list_remote_objects
# ---------------------------------------------------------------------------


def test_list_remote_objects_paginates_and_normalizes_prefix(monkeypatch):
    last_modified = datetime(2026, 1, 1, tzinfo=timezone.utc)
    pages = [
        {
            "Contents": [
                {"Key": "fonts/a.woff2", "Size": 4, "LastModified": last_modified},
                {"Key": "fonts/b.json", "Size": 8, "LastModified": last_modified},
            ]
        },
        {"Contents": [{"Key": "fonts/sub/c.bin", "Size": 2}]},  # no LastModified
        {},  # empty trailing page
    ]
    monkeypatch.setattr(_cdn_s3, "_s3_upload_client", lambda: _FakeListClient(pages))

    objects = list_remote_objects("fonts")  # no trailing slash -> normalized

    assert objects["fonts/a.woff2"] == RemoteObject(4, last_modified.timestamp())
    assert objects["fonts/b.json"] == RemoteObject(8, last_modified.timestamp())
    # Missing LastModified falls back to 0.0 so a newer local file re-uploads.
    assert objects["fonts/sub/c.bin"] == RemoteObject(2, 0.0)


# ---------------------------------------------------------------------------
# sync_directory — diff decision logic (size + mtime)
# ---------------------------------------------------------------------------


def test_sync_directory_uploads_new_changed_and_newer_skips_unchanged(
    tmp_path, monkeypatch
):
    local_dir = _build_sync_dir(tmp_path)
    calls, fake_upload = _record_uploads()
    monkeypatch.setattr(_cdn_s3, "upload_file", fake_upload)

    # a.woff2: same size, remote newer -> skip.
    # c.bin: size differs -> upload.
    # 一.svg: same size but remote older -> upload (mtime signal).
    future = 9_999_999_999.0
    monkeypatch.setattr(
        _cdn_s3,
        "list_remote_objects",
        lambda prefix: {
            "assets/a.woff2": RemoteObject(4, future),
            "assets/sub/c.bin": RemoteObject(999, future),
            "assets/一.svg": RemoteObject(4, 0.0),
        },
    )

    sync_directory(local_dir, "assets", "public, immutable", dry_run=False)

    keys = sorted(key for _, key, _, _ in calls)
    assert "assets/b.json" in keys  # missing remotely
    assert "assets/sub/c.bin" in keys  # size differs
    assert "assets/一.svg" in keys  # local newer than remote
    assert "assets/a.woff2" not in keys  # unchanged -> skipped
    assert "assets/README.md" not in keys  # README always skipped
    assert all(cc == "public, immutable" for _, _, cc, _ in calls)
    assert all(dry is False for _, _, _, dry in calls)


def test_sync_directory_same_size_newer_local_is_reuploaded(tmp_path, monkeypatch):
    # The size-only regression: a same-size content edit must still upload via
    # the mtime signal, or the CDN would serve stale content.
    local_dir = tmp_path / "d"
    local_dir.mkdir()
    target = local_dir / "x.json"
    target.write_bytes(b"exact-8")  # 7 bytes
    calls, fake_upload = _record_uploads()
    monkeypatch.setattr(_cdn_s3, "upload_file", fake_upload)
    monkeypatch.setattr(
        _cdn_s3,
        "list_remote_objects",
        lambda prefix: {"d/x.json": RemoteObject(7, 0.0)},  # same size, ancient remote
    )

    sync_directory(local_dir, "d", "immutable", dry_run=False)

    assert [key for _, key, _, _ in calls] == ["d/x.json"]


def test_sync_directory_dry_run_does_nothing_offline(tmp_path, monkeypatch):
    # The deploy orchestrator prints a per-directory header itself; in dry-run
    # sync_directory must neither walk the (100k+) local tree nor list remote,
    # so a dry-run preview stays instant and offline.
    local_dir = _build_sync_dir(tmp_path)
    calls, fake_upload = _record_uploads()
    monkeypatch.setattr(_cdn_s3, "upload_file", fake_upload)

    def fail_list(prefix: str) -> dict[str, RemoteObject]:
        raise AssertionError("dry-run must not list remote objects")

    monkeypatch.setattr(_cdn_s3, "list_remote_objects", fail_list)

    sync_directory(local_dir, "assets", "public, immutable", dry_run=True)

    assert calls == []


def test_sync_directory_cjk_key_round_trips(tmp_path, monkeypatch):
    # CJK filenames (kanji 一.svg) must map to unchanged S3 keys; this is the
    # boto3-path equivalent of the _UNSAFE_KEY_CHARS guard that the aws-CLI
    # path needs.
    local_dir = tmp_path / "kanji_animations"
    local_dir.mkdir()
    (local_dir / "一.svg").write_bytes(b"<svg/>")
    calls, fake_upload = _record_uploads()
    monkeypatch.setattr(_cdn_s3, "upload_file", fake_upload)
    monkeypatch.setattr(_cdn_s3, "list_remote_objects", lambda prefix: {})

    sync_directory(local_dir, "kanji_animations", "immutable", dry_run=False)

    assert [key for _, key, _, _ in calls] == ["kanji_animations/一.svg"]


def test_sync_directory_uses_normalized_prefix_key(tmp_path, monkeypatch):
    # A prefix with a trailing slash must not double-slash the key.
    local_dir = tmp_path / "d"
    local_dir.mkdir()
    (local_dir / "x.woff2").write_bytes(b"data")
    calls, fake_upload = _record_uploads()
    monkeypatch.setattr(_cdn_s3, "upload_file", fake_upload)
    monkeypatch.setattr(_cdn_s3, "list_remote_objects", lambda prefix: {})

    sync_directory(local_dir, "fonts/", "immutable", dry_run=False)

    assert [key for _, key, _, _ in calls] == ["fonts/x.woff2"]


# ---------------------------------------------------------------------------
# Credential-source contract: env credentials override the local profile
# (ADR-041) — applies to every script built on this transport.
# ---------------------------------------------------------------------------


class _FakeSession:
    """Records how the boto3 Session was constructed."""

    profile_name: str | None = None
    client_kwargs: dict[str, object] = {}

    def __init__(self, profile_name: str | None = None) -> None:
        _FakeSession.profile_name = profile_name

    def client(self, _service: str, **kwargs: object) -> str:
        _FakeSession.client_kwargs = kwargs
        return "fake-s3-client"


@pytest.fixture
def _fresh_client(monkeypatch):
    # The real _s3_client is a module singleton; reset it so each case
    # exercises Session construction itself.
    monkeypatch.setattr(_cdn_s3, "_s3_client", None)
    monkeypatch.setattr("boto3.Session", _FakeSession)
    yield
    _FakeSession.profile_name = None
    _FakeSession.client_kwargs = {}


def test_env_credentials_take_precedence_over_profile(monkeypatch, _fresh_client):
    monkeypatch.setenv("AWS_ACCESS_KEY_ID", "tid_ci_key")
    client = _cdn_s3._s3_upload_client()

    assert client == "fake-s3-client"
    # Env branch: Session() without an explicit profile — the boto3 env
    # credential chain owns resolution.
    assert _FakeSession.profile_name is None


def test_local_profile_used_when_no_env_credentials(monkeypatch, _fresh_client):
    monkeypatch.delenv("AWS_ACCESS_KEY_ID", raising=False)
    _cdn_s3._s3_upload_client()

    assert _FakeSession.profile_name == _cdn_s3.S3_PROFILE
    kwargs = _FakeSession.client_kwargs
    assert kwargs["endpoint_url"] == _cdn_s3.S3_ENDPOINT


# ---------------------------------------------------------------------------
# upload_file — chunk_size / checksum_algorithm (ADR-041 release uploads)
# ---------------------------------------------------------------------------


def test_upload_file_passes_chunk_size_config_and_checksum(tmp_path, monkeypatch):
    local = tmp_path / "Origa_x64-setup.exe"
    local.write_bytes(b"x" * 100)
    fake = _FakeUploadClient()
    monkeypatch.setattr(_cdn_s3, "_s3_upload_client", lambda: fake)

    chunk = 8 * 1024 * 1024
    upload_file(
        local,
        "releases/v1.2.3/Origa_x64-setup.exe",
        "public, max-age=300, must-revalidate",
        dry_run=False,
        chunk_size=chunk,
        checksum_algorithm="SHA256",
    )

    call = fake.calls[0]
    assert call["ExtraArgs"]["ChecksumAlgorithm"] == "SHA256"
    # The actual TransferConfig handed to the client — not just the argument
    # — must use the requested chunk size.
    assert call["Config"] is _cdn_s3._transfer_config(chunk)
    assert call["Config"].multipart_chunksize == chunk


def test_upload_file_retries_once_without_checksum_on_store_rejection(
    tmp_path, monkeypatch, capsys
):
    # S3-compatible stores may reject the checksum extension on multipart
    # creation; the upload must fall back to a checksum-less retry rather
    # than aborting the release. A wide catch is deliberate — a failure
    # unrelated to checksums simply fails the retry identically.
    from botocore.exceptions import ClientError

    local = tmp_path / "Origa_x64-setup.exe"
    local.write_bytes(b"x" * 10)
    err = ClientError(
        {"Error": {"Code": "NotImplemented", "Message": "checksum unsupported"}},
        "CreateMultipartUpload",
    )
    flaky = _FlakyUploadClient(err)
    monkeypatch.setattr(_cdn_s3, "_s3_upload_client", lambda: flaky)

    upload_file(
        local,
        "releases/latest/Origa_x64-setup.exe",
        "no-cache",
        dry_run=False,
        checksum_algorithm="SHA256",
    )

    assert len(flaky.calls) == 2
    assert flaky.calls[0]["ExtraArgs"]["ChecksumAlgorithm"] == "SHA256"
    assert "ChecksumAlgorithm" not in flaky.calls[1]["ExtraArgs"]
    assert "retrying without checksum" in capsys.readouterr().err


def test_upload_file_catches_boto3_flavored_upload_error(tmp_path, monkeypatch, capsys):
    # Live regression (2026-08-22 CI run): T3 answered UploadPart with
    # InternalError; boto3 re-raised it as boto3.exceptions.S3UploadFailedError
    # — a DIFFERENT class from s3transfer.exceptions.S3UploadFailedError. The
    # old catch tuple missed it and the operator got a raw traceback instead
    # of the retry (checksum case) or the keyed error message.
    from boto3.exceptions import S3UploadFailedError as Boto3Flavor

    local = tmp_path / "Origa_x64-setup.exe"
    local.write_bytes(b"x" * 10)
    err = Boto3Flavor("Failed to upload x: InternalError on UploadPart")
    flaky = _FlakyUploadClient(err)
    monkeypatch.setattr(_cdn_s3, "_s3_upload_client", lambda: flaky)

    upload_file(
        local,
        "releases/latest/Origa_x64-setup.exe",
        "no-cache",
        dry_run=False,
        checksum_algorithm="SHA256",
    )

    # Now caught: the retry (without checksum) fires and succeeds.
    assert len(flaky.calls) == 2
    assert "ChecksumAlgorithm" not in flaky.calls[1]["ExtraArgs"]
    assert "retrying without checksum" in capsys.readouterr().err


def test_upload_file_boto3_flavor_without_checksum_fails_with_key(
    tmp_path, monkeypatch, capsys
):
    # The same boto3-flavored failure on a checksum-less upload must exit
    # non-zero with the offending key, never a raw traceback.
    from boto3.exceptions import S3UploadFailedError as Boto3Flavor

    local = tmp_path / "Origa_x64-setup.exe"
    local.write_bytes(b"x" * 10)
    err = Boto3Flavor("Failed to upload x: InternalError on UploadPart")
    monkeypatch.setattr(_cdn_s3, "_s3_upload_client", lambda: _FakeUploadClient(err))

    with pytest.raises(SystemExit) as exc:
        upload_file(local, "releases/latest/Origa_x64-setup.exe", "no-cache", dry_run=False)

    assert exc.value.code == 1
    assert "releases/latest/Origa_x64-setup.exe" in capsys.readouterr().err


class _FlakyUploadClient:
    """Fails the first upload_file call, records every call."""

    def __init__(self, exc: BaseException) -> None:
        self.calls: list[dict[str, object]] = []
        self._exc = exc

    def upload_file(self, **kwargs: object) -> None:
        # Record a copy: defensive against any future production path that
        # reuses/mutates the ExtraArgs dict between attempts.
        recorded = {**kwargs}
        extra_args = recorded.get("ExtraArgs")
        if isinstance(extra_args, dict):
            recorded["ExtraArgs"] = dict(extra_args)
        self.calls.append(recorded)
        if len(self.calls) == 1:
            raise self._exc


# ---------------------------------------------------------------------------
# stat_object — boto3 HEAD (Linux-CI-safe, unlike the pwsh head_object)
# ---------------------------------------------------------------------------


class _FakeHeadClient:
    """Serves one canned head_object response or raises."""

    def __init__(
        self,
        response: dict[str, object] | None = None,
        exc: BaseException | None = None,
    ) -> None:
        self.calls: list[dict[str, object]] = []
        self._response = response or {}
        self._exc = exc

    def head_object(self, **kwargs: object) -> dict[str, object]:
        self.calls.append(kwargs)
        if self._exc is not None:
            raise self._exc
        return self._response


def test_stat_object_returns_metadata_and_checksum_mode(monkeypatch):
    fake = _FakeHeadClient(
        {
            "CacheControl": "no-cache",
            "ContentLength": 58_372_366,
            "ChecksumSHA256": "AbCd+/==",
        }
    )
    monkeypatch.setattr(_cdn_s3, "_s3_upload_client", lambda: fake)

    metadata = _cdn_s3.stat_object("releases/latest/Origa_x64-setup.exe")

    assert metadata == _cdn_s3.ObjectMetadata(
        cache_control="no-cache",
        content_length=58_372_366,
        checksum_sha256="AbCd+/==",
    )

    checksum_meta = _cdn_s3.stat_object(
        "releases/latest/Origa_x64-setup.exe", with_checksum=True
    )
    assert checksum_meta is not None
    assert checksum_meta.checksum_sha256 == "AbCd+/=="
    assert fake.calls[-1]["ChecksumMode"] == "ENABLED"
    assert "ChecksumMode" not in fake.calls[0]


def test_stat_object_returns_none_with_warning_on_error(monkeypatch, capsys):
    from botocore.exceptions import ClientError

    err = ClientError({"Error": {"Code": "404", "Message": "NoSuchKey"}}, "HeadObject")
    monkeypatch.setattr(_cdn_s3, "_s3_upload_client", lambda: _FakeHeadClient(exc=err))

    assert _cdn_s3.stat_object("releases/v9.9.9/missing.exe") is None
    assert "WARNING" in capsys.readouterr().err
