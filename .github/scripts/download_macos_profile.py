#!/usr/bin/env python3
"""Download a Mac App Store provisioning profile and generate merged entitlements.

Generates an ES256 JWT from the same .p8 API key used by xcrun altool,
calls GET /v1/profiles?filter[profileType]=MAC_APP_STORE, decodes the
base64 profileContent, and writes:
  1. embedded.provisionprofile (for .app/Contents/)
  2. Merged entitlements plist (app entitlements + profile entitlements)

Zero external dependencies — uses only Python stdlib + openssl (for
ECDSA signing and CMS decryption).

Usage:
    APPLE_API_KEY_PATH=path/to/AuthKey.p8 \\
    APPLE_API_KEY=KEYID123ABC \\
    APPLE_API_ISSUER=12345678-abcd-... \\
    python3 download_macos_profile.py \\
        --output-profile embedded.provisionprofile \\
        --output-entitlements Entitlements.merged.plist \\
        --app-entitlements tauri/Entitlements.plist \\
        [bundle_id]

Exit codes:
    0 — profile downloaded and entitlements merged
    1 — no matching profile found / API error
"""

from __future__ import annotations

import argparse
import base64
import json
import os
import plistlib
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

def fetch_mac_profile(
    jwt_token: str, bundle_id: str
) -> tuple[bytes, str] | None:
    """Fetch MAC_APP_STORE profile, return (profile_bytes, profile_name).

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
            return base64.b64decode(content), name
        if content:
            print(
                f"WARNING: profile '{name}' matches but state is {state}, skipping",
                file=sys.stderr,
            )

    return None


# ---------------------------------------------------------------------------
# Entitlements extraction & merge
# ---------------------------------------------------------------------------

def extract_profile_entitlements(profile_path: str) -> dict:
    """Extract the Entitlements dict from a .provisionprofile file.

    Uses `security cms -D` (macOS) to strip the CMS wrapper and reveal
    the inner plist, then parses it with plistlib.
    """
    result = subprocess.run(
        ["security", "cms", "-D", "-i", profile_path],
        capture_output=True,
        check=True,
    )
    plist_data = plistlib.loads(result.stdout)
    return plist_data.get("Entitlements", {})


def merge_entitlements(
    app_entitlements_path: str, profile_entitlements: dict
) -> dict:
    """Merge app entitlements with profile entitlements.

    App entitlements take precedence for keys that exist in both.
    Profile-only entitlements (com.apple.application-identifier,
    com.apple.developer.team-identifier, keychain-access-groups, etc.)
    are added.
    """
    with open(app_entitlements_path, "rb") as f:
        app_entitlements = plistlib.load(f)

    merged = dict(profile_entitlements)
    merged.update(app_entitlements)
    return merged


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main() -> int:
    parser = argparse.ArgumentParser(
        description="Download Mac App Store provisioning profile"
    )
    parser.add_argument(
        "bundle_id", nargs="?", default="net.uwuwu.origa", help="Bundle ID"
    )
    parser.add_argument(
        "--output-profile", default="embedded.provisionprofile",
        help="Output path for .provisionprofile"
    )
    parser.add_argument(
        "--output-entitlements", default=None,
        help="Output path for merged entitlements plist"
    )
    parser.add_argument(
        "--app-entitlements", default=None,
        help="App's Entitlements.plist to merge with profile entitlements"
    )
    args = parser.parse_args()

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

    if not os.path.isfile(key_path):
        print(f"ERROR: key file not found: {key_path}", file=sys.stderr)
        return 1

    print(f"Generating ES256 JWT (key_id={key_id}, issuer={issuer_id[:8]}…)")
    jwt_token = create_es256_jwt(key_path, key_id, issuer_id)

    print(f"Fetching MAC_APP_STORE profiles for {args.bundle_id}…")
    result = fetch_mac_profile(jwt_token, args.bundle_id)
    if result is None:
        print(
            f"ERROR: No active MAC_APP_STORE profile found for {args.bundle_id}.\n"
            "Create one at https://developer.apple.com/account/resources/profiles "
            "(Platform: macOS, Type: App Store).",
            file=sys.stderr,
        )
        return 1

    profile_bytes, profile_name = result

    with open(args.output_profile, "wb") as f:
        f.write(profile_bytes)

    print(
        f"✅ Downloaded '{profile_name}' ({len(profile_bytes)} bytes) "
        f"→ {args.output_profile}"
    )

    # Extract entitlements from profile and merge with app entitlements
    if args.output_entitlements and args.app_entitlements:
        profile_entitlements = extract_profile_entitlements(args.output_profile)
        merged = merge_entitlements(args.app_entitlements, profile_entitlements)

        with open(args.output_entitlements, "wb") as f:
            plistlib.dump(merged, f)

        print(f"✅ Merged entitlements → {args.output_entitlements}")
        print(f"   App keys:   {sorted(set(merged) - set(profile_entitlements))}")
        print(f"   Profile keys: {sorted(profile_entitlements)}")

    return 0


if __name__ == "__main__":
    sys.exit(main())
