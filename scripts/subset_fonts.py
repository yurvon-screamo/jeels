"""Generate self-hosted subset woff2 fonts for origa_ui.

Downloads pinned source fonts, extracts the CJK glyph corpus from the cdn/
content, subsets each font to the glyphs the app actually uses, and writes
content-hash-versioned woff2 files to cdn/fonts/. Hashing the filename gives
automatic CDN cache-bust on regeneration (PR #182 edge-poisoning class).

Run with fonttools available (woff2 flavor needs brotli):

    uv run --with "fonttools[woff]" scripts/subset_fonts.py

Outputs:
  - cdn/fonts/<logical>-<sha8>.woff2   (8 files)
  - end2end/cdn-manifest.txt            (FONTS-START/END block refreshed)

Source fonts are pinned by URL + SHA256; a mismatch aborts the run so an
upstream change forces a deliberate hash update (reproducibility, NFR-4).
"""

from __future__ import annotations

import hashlib
import subprocess
import sys
import zipfile
from collections.abc import Iterable
from dataclasses import dataclass
from pathlib import Path

PROJECT_ROOT = Path(__file__).resolve().parent.parent
CDN_DIR = PROJECT_ROOT / "cdn"
FONTS_OUT = CDN_DIR / "fonts"
CACHE_DIR = PROJECT_ROOT / "scripts" / ".font_cache"
CDN_MANIFEST = PROJECT_ROOT / "end2end" / "cdn-manifest.txt"

LATIN_UNICODES = "U+0000-007F,U+00A0-00FF,U+2000-206F,U+20A0-20CF,U+2100-214F"
# Cyrillic Supplement + Extended-A/B/C + Hryvnia sign. Cormorant Garamond and
# IBM Plex Mono ship these glyphs; subsetting them in keeps Cyrillic UI text
# from falling back to a system font (see ADR-030).
CYRILLIC_UNICODES = "U+0400-052F,U+1C80-1C88,U+20B4,U+2DE0-2DFF,U+A640-A69F"
# Latin-Extended (diacritics for Vietnamese/EA langs) added for IBM Plex Mono,
# which is the UI body font and must render Vietnamese loanwords / IPA.
LATIN_EXTENDED_UNICODES = "U+0100-017F,U+1E00-1EFF"
LATIN_CYRILLIC_UNICODES = f"{LATIN_UNICODES},{LATIN_EXTENDED_UNICODES},{CYRILLIC_UNICODES}"
# Ranges included wholesale into the corpus (small, always needed).
WHOLESALE_RANGES: tuple[range, ...] = (
    range(0x3000, 0x3040),   # CJK symbols and punctuation
    range(0x3040, 0x3100),   # Hiragana + katakana
    range(0xFF00, 0xFFEF + 1),  # Halfwidth/Fullwidth forms
    range(0x0020, 0x007F),   # ASCII
)
# Kanji ranges: only codepoints actually present in cdn/ content are kept,
# otherwise the full ~20k ideograph block would defeat subsetting.
KANJI_RANGES: tuple[tuple[int, int], ...] = (
    (0x3400, 0x4DC0),   # CJK Unified Ideographs Extension A
    (0x4E00, 0x9FFF + 1),  # CJK Unified Ideographs
    (0xF900, 0xFAFF + 1),  # CJK Compatibility Ideographs
)


def _is_kanji(codepoint: int) -> bool:
    return any(start <= codepoint < end for start, end in KANJI_RANGES)


@dataclass(frozen=True)
class Source:
    logical: str
    url: str
    sha256: str
    kind: str  # "cjk" | "latin_static" | "latin_variable"
    archive: str | None = None
    extract_glob: str | None = None


SOURCES: tuple[Source, ...] = (
    Source(
        logical="noto-sans-jp-400",
        url="https://github.com/notofonts/noto-cjk/releases/download/Sans2.004/16_NotoSansJP.zip",
        sha256="2bbdd2c20f30670b39ca735c96d75f1fdabdb348103e43b820cf17701fd22b18",
        kind="cjk",
        archive="NotoSansJP.zip",
        extract_glob="NotoSansJP-Regular.otf",
    ),
    Source(
        logical="cormorant-garamond",
        url="https://raw.githubusercontent.com/google/fonts/main/ofl/cormorantgaramond/CormorantGaramond%5Bwght%5D.ttf",
        sha256="b20b7d9626dd956b2c5e558692ad328b1f19e3275e2782db4fa07670d83f35e0",
        kind="latin_cyrillic_variable",
    ),
    Source(
        logical="cormorant-garamond-italic",
        url="https://raw.githubusercontent.com/google/fonts/main/ofl/cormorantgaramond/CormorantGaramond-Italic%5Bwght%5D.ttf",
        sha256="0f48ea6abb2084537854f7174c470991a463b13036309e3b50a81511611c530d",
        kind="latin_cyrillic_variable",
    ),
    Source(
        logical="ibm-plex-mono-300",
        url="https://raw.githubusercontent.com/google/fonts/main/ofl/ibmplexmono/IBMPlexMono-Light.ttf",
        sha256="780bcf65509d72a35ec114b57bcbe220dc6b77d8ea2e9b25e294be3c570c5025",
        kind="latin_cyrillic_vietnamese_static",
    ),
    Source(
        logical="ibm-plex-mono-400",
        url="https://raw.githubusercontent.com/google/fonts/main/ofl/ibmplexmono/IBMPlexMono-Regular.ttf",
        sha256="6a3412f058c7d8dfd9170c41e85ade48e5156ecb89356110ca57a0a27734af46",
        kind="latin_cyrillic_vietnamese_static",
    ),
    Source(
        logical="ibm-plex-mono-500",
        url="https://raw.githubusercontent.com/google/fonts/main/ofl/ibmplexmono/IBMPlexMono-Medium.ttf",
        sha256="a9b4c49bb299e05b5f6c481e7fb5e78943d2793249a0c8874ab574a2d1ea6755",
        kind="latin_cyrillic_vietnamese_static",
    ),
    Source(
        logical="ibm-plex-mono-400-italic",
        url="https://raw.githubusercontent.com/google/fonts/main/ofl/ibmplexmono/IBMPlexMono-Italic.ttf",
        sha256="3362fc791b0652193328b862c1c5f23a789bc7288b1617fa63302f88689a2a34",
        kind="latin_cyrillic_vietnamese_static",
    ),
)


def sha256_file(path: Path) -> str:
    hasher = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(65536), b""):
            hasher.update(chunk)
    return hasher.hexdigest()


def download(url: str, dest: Path) -> None:
    print(f"  downloading {url}")
    subprocess.run(
        ["curl", "-sSL", "--fail", "-o", str(dest), url],
        check=True,
    )


def ensure_source(src: Source) -> Path:
    """Return the local path to the extracted/ready-to-subset source font."""
    CACHE_DIR.mkdir(parents=True, exist_ok=True)
    if src.archive:
        archive_path = CACHE_DIR / src.archive
        if not archive_path.is_file() or sha256_file(archive_path) != src.sha256:
            download(src.url, archive_path)
        actual = sha256_file(archive_path)
        if actual != src.sha256:
            sys.exit(f"SHA256 mismatch for {src.archive}: {actual} != {src.sha256}")
        return extract_from_zip(archive_path, src.extract_glob or "*.otf")
    else:
        filename = src.url.rsplit("/", 1)[-1]
        font_path = CACHE_DIR / filename
        if not font_path.is_file() or sha256_file(font_path) != src.sha256:
            download(src.url, font_path)
        actual = sha256_file(font_path)
        if actual != src.sha256:
            sys.exit(f"SHA256 mismatch for {filename}: {actual} != {src.sha256}")
        return font_path


def extract_from_zip(archive_path: Path, pattern: str) -> Path:
    with zipfile.ZipFile(archive_path) as zf:
        matches = [n for n in zf.namelist() if Path(n).match(pattern)]
        if not matches:
            sys.exit(f"No file matching {pattern} in {archive_path.name}")
        target = CACHE_DIR / Path(matches[0]).name
        target.write_bytes(zf.read(matches[0]))
        return target


# Files whose kanji are reference data (radical -> every kanji sharing it),
# not text the UI renders; including them would bloat the subset to ~12k glyphs.
CORPUS_EXCLUDE: frozenset[str] = frozenset({"dictionary/radicals.json"})


def extract_cjk_corpus() -> set[int]:
    """Build the glyph set for Noto subsetting.

    Wholesale ranges (kana, punctuation, ASCII) are included in full since they
    are small and always relevant. Kanji are added only when actually present in
    the cdn/ content, so the subset stays bounded to the app's real vocabulary.
    Reference-only files (see CORPUS_EXCLUDE) are skipped.
    """
    corpus: set[int] = set()
    for r in WHOLESALE_RANGES:
        corpus.update(r)

    for json_path in CDN_DIR.rglob("*.json"):
        rel = json_path.relative_to(CDN_DIR).as_posix()
        if rel in CORPUS_EXCLUDE:
            continue
        try:
            text = json_path.read_text(encoding="utf-8")
        except (OSError, UnicodeDecodeError):
            continue
        for ch in text:
            if _is_kanji(ord(ch)):
                corpus.add(ord(ch))
    return corpus


def write_corpus_file(codepoints: Iterable[int]) -> Path:
    corpus_path = CACHE_DIR / "cjk_corpus.txt"
    chars = "".join(chr(c) for c in sorted(codepoints))
    corpus_path.write_text(chars, encoding="utf-8")
    return corpus_path


def unicode_for_kind(kind: str) -> str:
    """Pick the `--unicodes` range for a non-CJK source.

    Cormorant Garamond and IBM Plex Mono both ship Cyrillic (and IBM Plex Mono
    adds Latin-Extended for Vietnamese), so they share the Latin+Cyrillic range
    (see ADR-030). Plain Latin-only fonts would fall back to the base range.
    """
    if kind in ("latin_cyrillic_variable", "latin_cyrillic_vietnamese_static"):
        return LATIN_CYRILLIC_UNICODES
    return LATIN_UNICODES


def subset_one(src_path: Path, src: Source, corpus_path: Path) -> Path:
    """Run pyftsubset and return the path to the content-hashed output."""
    raw_out = FONTS_OUT / f"{src.logical}.raw.woff2"
    cmd = [sys.executable, "-m", "fontTools.subset", str(src_path)]
    if src.kind == "cjk":
        cmd += ["--text-file=" + str(corpus_path)]
    else:
        cmd += ["--unicodes=" + unicode_for_kind(src.kind)]
    cmd += [
        "--flavor=woff2",
        "--layout-features=*",
        "--output-file=" + str(raw_out),
    ]
    subprocess.run(cmd, check=True)
    return raw_out


def hash_rename(raw_path: Path) -> Path:
    digest = sha256_file(raw_path)[:8]
    final = raw_path.with_name(f"{raw_path.name.removesuffix('.raw.woff2')}-{digest}.woff2")
    raw_path.replace(final)
    return final


def update_cdn_manifest(font_relative: Iterable[str]) -> None:
    start_marker = "# FONTS-START"
    end_marker = "# FONTS-END"
    original = (
        CDN_MANIFEST.read_text(encoding="utf-8").splitlines()
        if CDN_MANIFEST.is_file()
        else []
    )

    marker_count = sum(
        1 for line in original if line.strip() in (start_marker, end_marker)
    )
    if marker_count == 1:
        sys.exit(
            f"{CDN_MANIFEST} has a lone FONTS marker; remove the stray marker or "
            "pair START/END before regenerating."
        )

    outside: list[str] = []
    in_block = False
    for line in original:
        if line.strip() == start_marker:
            in_block = True
            continue
        if line.strip() == end_marker:
            in_block = False
            continue
        if not in_block:
            outside.append(line)

    block = [start_marker, *sorted(font_relative), end_marker]
    body = "\n".join(outside).rstrip()
    content = body + "\n\n" + "\n".join(block) + "\n" if body else "\n".join(block) + "\n"
    CDN_MANIFEST.write_text(content, encoding="utf-8")


def main() -> None:
    FONTS_OUT.mkdir(parents=True, exist_ok=True)

    corpus_path = write_corpus_file(extract_cjk_corpus())

    print("Subsetting fonts:")
    produced: list[str] = []
    for src in SOURCES:
        try:
            source_path = ensure_source(src)
        except (subprocess.CalledProcessError, OSError):
            # Transient network failure (e.g. release-assets.githubusercontent.com
            # blocked). Preserve any existing woff2 for this logical name so the
            # CDN/font registry stays consistent; the source can be re-fetched
            # later from a network with access.
            existing = sorted(FONTS_OUT.glob(f"{src.logical}-*.woff2"))
            if existing:
                print(f"  SKIP {src.logical}: source unavailable, keeping {existing[0].name}")
                produced.append(f"fonts/{existing[0].name}")
            else:
                print(f"  SKIP {src.logical}: source unavailable and no existing woff2")
            continue
        # Remove only the stale woff2 for this logical before regenerating.
        for stale in FONTS_OUT.glob(f"{src.logical}-*.woff2"):
            stale.unlink()
        raw = subset_one(source_path, src, corpus_path)
        final = hash_rename(raw)
        size_kb = final.stat().st_size // 1024
        print(f"  {final.name}  ({size_kb} KB)")
        produced.append(f"fonts/{final.name}")

    update_cdn_manifest(produced)
    verify_glyph_coverage()
    print(f"\nDone: {len(produced)} fonts in {FONTS_OUT}")
    print(f"Corpus: {len(corpus_path.read_text(encoding='utf-8'))} glyphs")


# Glyphs the font set MUST render: the bug-report kanji (must use Japanese
# forms), representative kana, and CJK punctuation. A failed assertion means
# the corpus scan or subset dropped coverage the app depends on.
MUST_HAVE = "食海語難挨拶あア。、「」ー"
# Cyrillic glyphs the UI body font MUST render now that Cormorant + IBM Plex
# Mono subsets include Cyrillic (ADR-030). A failed assertion means the subset
# unicode range regressed and Cyrillic UI text would fall back to a system font.
CYRILLIC_MUST_HAVE = "АаБбВвГгДдЕеЁёЖжЗзИиЙйКкЛлМмНнОоПпРрСсТтУуФфХхЦцЧчШшЩщЪъЫыЬьЭэЮюЯя"


def verify_glyph_coverage() -> None:
    from fontTools.ttLib import TTFont

    noto = next(FONTS_OUT.glob("noto-sans-jp-400-*.woff2"))
    cmap = TTFont(noto).getBestCmap()
    missing = [c for c in MUST_HAVE if ord(c) not in cmap]
    if missing:
        sys.exit(f"Coverage regression: {missing} missing from {noto.name}")
    print(f"Coverage OK: {len(MUST_HAVE)} must-have glyphs present in {noto.name}")

    mono = next(FONTS_OUT.glob("ibm-plex-mono-400-*.woff2"))
    mono_cmap = TTFont(mono).getBestCmap()
    cyrillic_missing = [c for c in CYRILLIC_MUST_HAVE if ord(c) not in mono_cmap]
    if cyrillic_missing:
        sys.exit(
            f"Cyrillic coverage regression: {cyrillic_missing} missing from {mono.name}"
        )
    print(
        f"Cyrillic coverage OK: {len(CYRILLIC_MUST_HAVE)} must-have glyphs present in {mono.name}"
    )


if __name__ == "__main__":
    main()
