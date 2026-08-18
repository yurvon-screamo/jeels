#!/usr/bin/env python
"""Build the Origa tokenizer dictionary (SudachiDict small+core + extras).

Pipeline (mirrors lindera docs/src/sudachidict.md):
  1. small_lex.csv + core_lex.csv + EXTRA_WORDS rows  ->  merged lexicon
  2. unk.def gets a display-surface placeholder column so unknown-word
     details land at the same indices as dictionary words
  3. `lindera build` with the 19-column schema metadata

The runtime user dictionary was removed: extra vocabulary is baked into the
system lexicon here (same schema, same POS indices — Token::get works).

Usage:
  python scripts/build_sudachidict.py --src <dir with lex csvs> --lindera <path to lindera.exe> --out <dest>

Source CSVs (20260723): http://sudachi.s3-website-ap-northeast-1.amazonaws.com/sudachidict-raw/20260723/
"""

import argparse
import shutil
import subprocess
import sys
from pathlib import Path

# Extra vocabulary absent from small/core lex. 19-column rows in SudachiDict
# order: surface,left,right,cost,display,pos,sub1,sub2,sub3,ctype,cform,
#        reading,normalized_form,dict_form_id,split_mode,split_a,split_b,
#        word_structure,synonym_group_ids
# left/right context ids: 連体詞=5979/5979, 感動詞=5688/5688 (same-POS rows in core_lex).
RENDAISHI = "5979,5979"
KANDOUSHI = "5688,5688"
MEISHI = "5139,5139"   # 普通名詞 contexts (same as KANJI unk rows)

def row(surface, pos, reading, ctx, cost=4000):
    return (f"{surface},{ctx},{cost},{surface},{pos},*,*,*,*,*,"
            f"{reading},{surface},*,A,*,*,*,*")

EXTRA_WORDS = [
    # 連体詞 compounds UniDic split (そう+いう); learners see them as one word
    *[row(w, "連体詞", r, RENDAISHI) for w, r in [
        ("そういう", "ソウイウ"), ("こういう", "コウイウ"), ("ああいう", "アアイウ"),
        ("という", "トイウ"), ("ていう", "テイウ"), ("っていう", "ッテイウ"),
        ("なんていう", "ナンテイウ"), ("といった", "トイッタ"), ("そういった", "ソウイッタ"),
        ("そういうふうに", "ソウイウフウニ"), ("こういうふうに", "コウイウフウニ"),
        ("ああいうふうに", "アアイウフウニ"), ("どういうふうに", "ドウイウフウニ"),
        ("そうした", "ソウシタ"), ("こうした", "コウシタ"), ("ちょっとした", "チョットシタ"),
        ("ふとした", "フトシタ"), ("れっきとした", "レッキトシタ"),
        ("他の", "タノ"), ("当の", "トウノ"), ("これらの", "コレラノ"),
        ("それらの", "ソレラノ"), ("あれらの", "アレラノ"), ("なんらかの", "ナンラカノ"),
        ("何らかの", "ナンラカノ"), ("まさかの", "マサカノ"), ("あまりの", "アマリノ"),
        ("たっての", "タッテノ"), ("しかるべき", "シカルベキ"), ("然るべき", "シカルベキ"),
        ("恐るべき", "オソルベキ"), ("見知らぬ", "ミシラヌ"),
        ("ありとあらゆる", "アリトアラユル"), ("聖なる", "セイナル"),
        ("確固たる", "カッコタル"), ("最たる", "サイタル"), ("微々たる", "ビビタル"),
        ("隠然たる", "インゼンタル"), ("輝ける", "カガヤケル"), ("そうゆう", "ソウユウ"),
    ]],
    row("やれやれ", "感動詞", "ヤレヤレ", KANDOUSHI),
    # rewrite Sudachi OOV placeholder rows (-1,-1,0) with real contexts
    row("ひとつ", "名詞", "ヒトツ", MEISHI),
    row("一つ", "名詞", "ヒトツ", MEISHI),
    # katakana slang spelling of かっこいい (manga/STT surface)
    row("カッコイイ", "形容詞", "カッコイイ", RENDAISHI),
    # STT katakana lexicon (speech-to-text output never uses kanji)
    row("ジドウシャ", "名詞", "ジドウシャ", MEISHI),
    # ジドーシャ already in core_lex
    row("ものか", "名詞", "モノカ", MEISHI),
]

METADATA = """{
  "name": "sudachidict",
  "encoding": "UTF-8",
  "default_word_cost": -10000,
  "default_left_context_id": 0,
  "default_right_context_id": 0,
  "default_field_value": "*",
  "flexible_csv": true,
  "skip_invalid_cost_or_id": true,
  "normalize_details": false,
  "dictionary_schema": {
    "fields": [
      "surface", "left_context_id", "right_context_id", "cost", "display_surface",
      "part_of_speech", "part_of_speech_subcategory_1", "part_of_speech_subcategory_2",
      "part_of_speech_subcategory_3", "conjugation_type", "conjugation_form",
      "reading", "normalized_form", "dictionary_form_id", "split_mode",
      "split_a", "split_b", "word_structure", "synonym_group_ids"
    ]
  },
  "user_dictionary_schema": { "fields": ["surface", "part_of_speech", "reading"] }
}"""


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--src", required=True, help="dir with *_lex.csv + unk.def + char.def")
    ap.add_argument("--lindera", required=True, help="path to lindera CLI executable")
    ap.add_argument("--out", required=True, help="destination dir for built dictionary")
    ap.add_argument("--work", default=None, help="scratch dir (default: <src>-build)")
    args = ap.parse_args()

    src = Path(args.src)
    work = Path(args.work) if args.work else src.parent / (src.name + "-build")
    out = Path(args.out)
    work.mkdir(parents=True, exist_ok=True)

    # 1) copy sources, appending extra rows to core_lex.csv
    for name in ("small_lex.csv", "core_lex.csv", "char.def", "matrix.def"):
        shutil.copy2(src / name, work / name)

    extra_path = work / "origa_extra_lex.csv"
    extra_path.write_text("\n".join(EXTRA_WORDS) + "\n", encoding="utf-8")
    print(f"extra rows: {len(EXTRA_WORDS)} -> {extra_path}")

    # 1b) rewrite Sudachi OOV placeholder rows (left_id=-1, right_id=-1,
    # cost=0) with the EXTRA rows so the segmenter actually picks them up.
    replacements = {}
    for extra in EXTRA_WORDS:
        cols = extra.split(",")
        replacements[cols[0]] = extra
    for name in ("core_lex.csv", "small_lex.csv"):
        path = work / name
        out_lines = []
        replaced = 0
        for line in path.read_text(encoding="utf-8").splitlines():
            surf = line.split(",", 1)[0]
            if surf in replacements and ",-1,-1,0," in line:
                out_lines.append(replacements[surf])
                replaced += 1
            else:
                out_lines.append(line)
        path.write_text("\n".join(out_lines) + "\n", encoding="utf-8")
        print(f"{name}: {replaced} OOV rows rewritten")

    # 2) align unk.def: insert display_surface placeholder after cost column
    unk_src = src / "unk.def"
    unk_dst = work / "unk.def"
    with unk_src.open(encoding="utf-8") as f, unk_dst.open("w", encoding="utf-8", newline="") as g:
        for line in f:
            line = line.rstrip("\r\n")
            if not line:
                continue
            cols = line.split(",")
            # [category,left,right,cost,pos,...] -> insert "*" after cost
            aligned = cols[:4] + ["*"] + cols[4:]
            g.write(",".join(aligned) + "\n")
    print(f"unk.def aligned -> {unk_dst}")

    # 3) metadata
    meta_path = work / "metadata.json"
    meta_path.write_text(METADATA, encoding="utf-8")

    # 4) build
    cmd = [args.lindera, "build", "--src", str(work), "--dest", str(out),
           "--metadata", str(meta_path)]
    print("+", " ".join(cmd))
    r = subprocess.run(cmd)
    if r.returncode != 0:
        sys.exit(f"lindera build failed with code {r.returncode}")
    print(f"built -> {out}")


if __name__ == "__main__":
    main()
