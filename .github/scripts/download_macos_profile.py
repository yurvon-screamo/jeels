#!/usr/bin/env python3
"""Download a Mac App Store provisioning profile via App Store Connect API.

Generates an ES256 JWT from the same .p8 API key used by xcrun altool,
calls GET /v1/profiles?filter[profileType]=MAC_APP_STORE, decodes the
base64 profileContent, and writes an embedded.provisionprofile file.

Zero external dependencies — uses only Python stdlib + openssl (for
ECDSA signing, avoiding the need for PyJWT/cryptography).

Usage:
    APPLE_API_KEY_PATH=path/to/AuthKey.p8 \
    APPLE_API_KEY=KEYID123ABC \
    APPLE_API_ISSUER=12345678-abcd-... \
    python3 download_macos_profile.py <output_path> [bundle_id]

Exit codes:
    0 — profile downloaded successfully
    1 — no matching profile found / API error
"""

from __future__ import annotations

import base64
import json
import os
import subprocess
import sys
import tempfile
import time
import urllib.error
import urllib.parse
import urllib.request

ASC_API_BASE = "https://api.appstoreconnect.apple.com/v1"


# ---------------------------------------------------------------------------
# JWT (ES256) — pure stdlib + openssl
# ---------------------------------------------------------------------------

def _b64url(data: bytes) -> str:
    """Base64url encode without trailing '=' padding."""
    return base64.urlsafe_b64encode(data).rstrip(b"=").decode("ascii")


def _der_to_raw_ecdsa(der: bytes) -> bytes:
    """Convert a DER-encoded ECDSA signature to raw R‖S (64 bytes for P-256)."""
    idx = 0
    if der[idx] != 0x30:
        raise ValueError("Invalid DER: expected SEQUENCE tag 0x30")
    idx += 1
    total_len = der[idx]
    if total_len & 0x80:
        nbytes = total_len & 0x7F
        idx += 1
        idx += nbytes  # skip long-form length
    else:
        idx += 1

    # R
    if der[idx] != 0x02:
        raise ValueError("Invalid DER: expected INTEGER for R")
    idx += 1
    r_len = der[idx]
    idx += 1
    r = int.from_bytes(der[idx : idx + r_len], "big")
    idx += r_len

    # S
    if der[idx] != 0x02:
        raise ValueError("Invalid DER: expected INTEGER for S")
    idx += 1
    s_len = der[idx]
    idx += 1
    s = int.from_bytes(der[idx : idx + s_len], "big")

    return r.to_bytes(32, "big") + s.to_bytes(32, "big")


def create_es256_jwt(key_path: str, key_id: str, issuer_id: str) -> str:
    """Create an ES256-signed JWT for App Store Connect API."""
    header = {"alg": "ES256", "kid": key_id, "typ": "JWT"}
    now = int(time.time())
    payload = {
        "iss": issuer_id,
        "iat": now,
        "exp": now + 900,  # 15 min (Apple max: 20 min)
        "aud": "appstoreconnect-v1",
    }

    header_b64 = _b64url(json.dumps(header, separators=(",", ":")).encode())
    payload_b64 = _b64url(json.dumps(payload, separators=(",", ":")).encode())
    signing_input = f"{header_b64}.{payload_b64}".encode()

    # Sign with openssl (ECDSA-SHA256 → DER)
    with tempfile.NamedTemporaryFile(delete=False) as tmp:
        tmp.write(signing_input)
        tmp_path = tmp.name

    try:
        result = subprocess.run(
            ["openssl", "dgst", "-sha256", "-sign", key_path, tmp_path],
            capture_output=True,
            check=True,
        )
    except subprocess.CalledProcessError as exc:
        raise RuntimeError(
            f"openssl ECDSA signing failed: {exc.stderr.decode().strip()}"
        ) from exc
    finally:
        os.unlink(tmp_path)

    raw_sig = _der_to_raw_ecdsa(result.stdout)
    return f"{header_b64}.{payload_b64}.{_b64url(raw_sig)}"


# ---------------------------------------------------------------------------
# App Store Connect API
# ---------------------------------------------------------------------------

def fetch_mac_profiles(
    jwt_token: str, bundle_id: str
) -> tuple[str, str] | None:
    """Fetch MAC_APP_STORE profiles, return (profileContent_b64, profile_name).

    Returns None if no matching profile is found.
    """
    params = urllib.parse.urlencode(
        {
            "filter[profileType]": "MAC_APP_STORE",
            "include": "bundleId",
            "limit": "20",
        }
    )
    url = f"{ASC_API_BASE}/profiles?{params}"

    req = urllib.request.Request(url)
    req.add_header("Authorization", f"Bearer {jwt_token}")
    req.add_header("Accept", "application/json")

    try:
        with urllib.request.urlopen(req, timeout=30) as resp:
            data = json.loads(resp.read())
    except urllib.error.HTTPError as exc:
        body = exc.read().decode("utf-8", errors="replace")
        print(f"API error {exc.code}: {body}", file=sys.stderr)
        raise

    # Build bundle_id ID → identifier lookup from included
    bundle_lookup: dict[str, str] = {}
    for inc in data.get("included", []):
        if inc.get("type") == "bundleIds":
            identifier = inc.get("attributes", {}).get("identifier", "")
            bundle_lookup[inc["id"]] = identifier

    # Find profile matching our bundle ID
    for profile in data.get("data", []):
        if profile.get("type") != "profiles":
            continue
        rel = profile.get("relationships", {}).get("bundleId", {})
        bid = rel.get("data", {}).get("id", "")
        if bundle_lookup.get(bid) != bundle_id:
            continue

        attrs = profile.get("attributes", {})
        content = attrs.get("profileContent")
        name = attrs.get("name", "unknown")
        state = attrs.get("profileState", "unknown")
        if content and state == "ACTIVE":
            return content, name
        if content:
            print(
                f"WARNING: profile '{name}' matches but state is {state}, skipping",
                file=sys.stderr,
            )

    return None


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main() -> int:
    key_path = os.environ.get("APPLE_API_KEY_PATH", "")
    key_id = os.environ.get("APPLE_API_KEY", "")
    issuer_id = os.environ.get("APPLE_API_ISSUER", "")

    if not all([key_path, key_id, issuer_id]):
        print(
            "ERROR: APPLE_API_KEY_PATH, APPLE_API_KEY, APPLE_API_ISSUER "
            "must all be set",
            file=sys.stderr,
        )
        return 1

    output_path = sys.argv[1] if len(sys.argv) > 1 else "embedded.provisionprofile"
    bundle_id = sys.argv[2] if len(sys.argv) > 2 else "net.uwuwu.origa"

    if not os.path.isfile(key_path):
        print(f"ERROR: key file not found: {key_path}", file=sys.stderr)
        return 1

    print(f"Generating ES256 JWT (key_id={key_id}, issuer={issuer_id[:8]}…)")
    jwt_token = create_es256_jwt(key_path, key_id, issuer_id)

    print(f"Fetching MAC_APP_STORE profiles for {bundle_id}…")
    result = fetch_mac_profiles(jwt_token, bundle_id)
    if result is None:
        print(
            f"ERROR: No active MAC_APP_STORE profile found for {bundle_id}.\n"
            "Create one at https://developer.apple.com/account/resources/profiles "
            "(Platform: macOS, Type: App Store).",
            file=sys.stderr,
        )
        return 1

    content_b64, profile_name = result
    profile_bytes = base64.b64decode(content_b64)

    with open(output_path, "wb") as f:
        f.write(profile_bytes)

    print(
        f"✅ Downloaded '{profile_name}' ({len(profile_bytes)} bytes) → {output_path}"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
