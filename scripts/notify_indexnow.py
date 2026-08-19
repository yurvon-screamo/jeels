"""Notify IndexNow that landing URLs were updated (ADR-038).

Submits every URL from the landing sitemap to the global IndexNow endpoint
(``api.indexnow.org/IndexNow``) after a stable release. Search engines that
participate in IndexNow (Yandex, Bing, Naver, Seznam, ...) then re-crawl the
submitted URLs ahead of their normal schedule.

The script is zero-dependency (Python standard library only) so the CI runner
can execute it directly after checkout.

Key handling: the IndexNow key is not a secret — it is publicly served at
``https://origa.uwuwu.net/<key>.txt`` to prove domain ownership. The key
constant must stay in sync with ``origa_landing/public/<key>.txt``; rotating
the key means updating both in the same commit.

Usage::

    python scripts/notify_indexnow.py            # parse sitemap, POST batches
    python scripts/notify_indexnow.py --dry-run  # print URLs, do not POST
    python scripts/notify_indexnow.py --sitemap other_sitemap.xml

Exit codes: 0 on success (HTTP 200/202 per batch), 1 on any failure (fetch,
parse, or a non-200/202 API response) so CI surfaces the breakage.
"""

from __future__ import annotations

import argparse
import json
import sys
import urllib.error
import urllib.request
import xml.etree.ElementTree as ET
from collections.abc import Iterator

INDEXNOW_ENDPOINT = "https://api.indexnow.org/IndexNow"
INDEXNOW_KEY = "e7825074-6888-4e03-a9ad-91459e4c9940"
INDEXNOW_HOST = "origa.uwuwu.net"
KEY_LOCATION = f"https://{INDEXNOW_HOST}/{INDEXNOW_KEY}.txt"

DEFAULT_SITEMAP_URL = f"https://{INDEXNOW_HOST}/sitemap.xml"

# IndexNow accepts up to 10 000 URLs per request; split far below that so a
# single batch stays a small payload regardless of sitemap growth.
BATCH_SIZE = 500

TIMEOUT_SECONDS = 30

# 200 = accepted, 202 = accepted pending key verification (first submission).
SUCCESS_STATUS_CODES = frozenset({200, 202})


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Submit sitemap URLs to IndexNow after a release.",
    )
    parser.add_argument(
        "--sitemap",
        default=DEFAULT_SITEMAP_URL,
        help=f"sitemap URL or local path (default: {DEFAULT_SITEMAP_URL})",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="print the extracted URLs without submitting anything",
    )
    return parser.parse_args()


def load_sitemap(source: str) -> str:
    """Return the sitemap XML text from a URL or a local file path."""
    if source.startswith(("http://", "https://")):
        print(f"Fetching sitemap: {source}")
        with urllib.request.urlopen(source, timeout=TIMEOUT_SECONDS) as response:
            return response.read().decode("utf-8")
    with open(source, encoding="utf-8") as file:
        return file.read()


def extract_urls(xml_text: str) -> list[str]:
    """Extract every ``<loc>`` URL from sitemap XML, preserving order."""
    root = ET.fromstring(xml_text)
    namespaces = {"sm": "http://www.sitemaps.org/schemas/sitemap/0.9"}
    urls = [loc.text.strip() for loc in root.findall(".//sm:loc", namespaces)]
    if not urls:
        raise ValueError("sitemap contains no <loc> URLs")
    return urls


def chunked(items: list[str], size: int) -> Iterator[list[str]]:
    """Yield successive ``size``-sized chunks of ``items``."""
    for start in range(0, len(items), size):
        yield items[start : start + size]


def submit_batch(urls: list[str]) -> int:
    """POST one URL batch to IndexNow; return the HTTP status code."""
    payload = {
        "host": INDEXNOW_HOST,
        "key": INDEXNOW_KEY,
        "keyLocation": KEY_LOCATION,
        "urlList": urls,
    }
    request = urllib.request.Request(
        INDEXNOW_ENDPOINT,
        data=json.dumps(payload).encode("utf-8"),
        headers={"Content-Type": "application/json; charset=utf-8"},
        method="POST",
    )
    try:
        with urllib.request.urlopen(request, timeout=TIMEOUT_SECONDS) as response:
            return response.status
    except urllib.error.HTTPError as error:
        return error.code


def main() -> int:
    args = parse_args()

    try:
        xml_text = load_sitemap(args.sitemap)
        urls = extract_urls(xml_text)
    except (OSError, ValueError, ET.ParseError) as error:
        print(f"error: failed to load sitemap: {error}", file=sys.stderr)
        return 1

    print(f"Extracted {len(urls)} URLs from sitemap")

    if args.dry_run:
        for url in urls:
            print(f"  {url}")
        print("dry-run: nothing submitted")
        return 0

    failures = 0
    for batch_number, batch in enumerate(chunked(urls, BATCH_SIZE), start=1):
        status = submit_batch(batch)
        if status in SUCCESS_STATUS_CODES:
            print(
                f"batch {batch_number}: submitted {len(batch)} URLs -> "
                f"HTTP {status}"
            )
        else:
            failures += 1
            print(
                f"error: batch {batch_number} ({len(batch)} URLs) rejected: "
                f"HTTP {status}",
                file=sys.stderr,
            )

    if failures:
        print(f"error: {failures} batch(es) failed", file=sys.stderr)
        return 1
    print("IndexNow notified successfully")
    return 0


if __name__ == "__main__":
    sys.exit(main())
