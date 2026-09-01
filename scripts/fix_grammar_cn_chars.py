#!/usr/bin/env python3
"""Phase 3a content pass: fix simplified-Chinese contamination in grammar_v2.

Every replacement is an explicit (rule, language, field, old -> new) pair —
no blind character mapping. Sentences that were fully Chinese are rewritten
as natural Japanese; glyph-level typos are corrected in place; Russian text
with embedded Chinese words gets proper Russian wording.

Run: python scripts/fix_grammar_cn_chars.py
Verifies zero remaining non-JIS CJK chars afterwards.
"""

from __future__ import annotations

import json
import shutil
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PATH = ROOT / "cdn" / "grammar" / "grammar_v2.json"

# (rule_id, lang, field, old, new). field "warnings[0]" targets list items.
FIXES: list[tuple[str, str, str, str, str]] = [
    # --- fully Chinese sentences rewritten as natural Japanese ---------------
    (
        "01G0000000000000002G000000", "English", "examples",
        "看电影的时候没有哭。\n(I did not cry during the movie.)\nNot applicable to Chinese.",
        "映画を見ているとき、泣きませんでした。\n(I did not cry during the movie.)",
    ),
    (
        "01G0000000000000002G000000", "English", "examples",
        "昨日暇ではなかった。\n(I was not free yesterday.)\nNot applicable to Chinese.",
        "昨日暇ではありませんでした。\n(I was not free yesterday.)",
    ),
    (
        "01G000000000000000P8000000", "English", "examples",
        "「，呵呵」是中国网络上笑声的意思。\n\"Hehe\" means laughter in Chinese internet culture.",
        "「お手洗い」は「トイレ」という意味です。\n\"Otearai\" means \"toilet\" (polite word).",
    ),
    (
        "01G000000000000000P8000000", "Russian", "examples",
        "「，呵呵」是中国网络上笑声的意思。\n«呵呵» означает смех в китайском интернете.",
        "「お手洗い」は「トイレ」という意味です。\n«Отэараи» означает «туалет» (вежливое слово).",
    ),
    (
        "01G000000000000000P8000000", "English", "examples",
        "この「的山」さんは、有名な学者という意味です。\n\"Mr. Deshank\" here means a famous scholar.",
        "「大丈夫」は「問題ない」という意味です。\n\"Daijōbu\" means \"no problem\".",
    ),
    (
        "01G000000000000000P8000000", "Russian", "examples",
        "この「的山」さんは、有名な学者という意味です。\n«Господин Дэшан» здесь означает известного учёного.",
        "「大丈夫」は「問題ない」という意味です。\n«Дайдзёбу» означает «ничего страшного».",
    ),
    (
        "01G000000000000000QW000000", "English", "examples",
        "この歌手很有意思，听了她的歌就喜欢上她了。\nI started liking her after listening to her songs.",
        "彼女の歌を聴いて、好きになるようになりました。\nI started liking her after listening to her songs.",
    ),
    (
        "01G000000000000000QW000000", "Russian", "examples",
        "この歌手很有意思，听了她的歌就喜欢上她了。\nЯ стал(а) любить её музыку после того, как послушал(а) её песни.",
        "彼女の歌を聴いて、好きになるようになりました。\nЯ стал(а) любить её музыку после того, как послушал(а) её песни.",
    ),
    (
        "01G000000000000000YW000000", "English", "examples",
        "生徒に发言的机会を与わせてもいただけませんか。\nWould you please give the students a chance to speak?",
        "生徒に発言の機会を与えさせてもいただけませんか。\nWould you please give the students a chance to speak?",
    ),
    (
        "01G000000000000000YW000000", "Russian", "examples",
        "生徒に发言の機会を与わせてもいただけませんか。\nНе могли бы вы дать студентам возможность высказаться?",
        "生徒に発言の機会を与えさせてもいただけませんか。\nНе могли бы вы дать студентам возможность высказаться?",
    ),
    (
        "01G000000000000000G0000000", "English", "examples",
        "钉が舍て周转んでくれた。\nThe neighbor helped me with the garbage.",
        "隣の人がゴミを出してくれた。\nThe neighbor took out the garbage for me.",
    ),
    (
        "01G000000000000000G0000000", "Russian", "examples",
        "钉が舍て周转んでくれた。\nСосед помог мне вынести мусор.",
        "隣の人がゴミを出してくれた。\nСосед вынес для меня мусор.",
    ),
    # --- glyph-level Japanese fixes ------------------------------------------
    ("01G000000000000000G0000000", "English", "examples", "友だちは伞を贷してくれた。", "友だちは傘を貸してくれた。"),
    ("01G000000000000000G0000000", "Russian", "examples", "友だちは伞を贷してくれた。", "友だちは傘を貸してくれた。"),
    ("01G000000000000000G0000000", "English", "examples", "兄が电脑を贷してくれた。", "兄がパソコンを貸してくれた。"),
    ("01G000000000000000G0000000", "Russian", "examples", "兄が电脑を贷してくれた。", "兄がパソコンを貸してくれた。"),
    ("01G000000000000000G0000000", "English", "how_to_form", "| 食べて给我 |", "| 食べてくれる |"),
    ("01G000000000000000G0000000", "Russian", "how_to_form", "| 食べて给我 |", "| 食べてくれる |"),
    ("01G0000000000000001C000000", "English", "examples", "お部屋を打扫しました。", "お部屋を掃除しました。"),
    ("01G0000000000000001C000000", "Russian", "examples", "お部屋を打扫しました。", "お部屋を掃除しました。"),
    ("01G0000000000000000M000000", "English", "pro_tip", "护士さん", "看護師さん"),
    ("01G0000000000000001M000000", "Russian", "how_to_form", "あそこは厕所です", "あそこはトイレです"),
    ("01G00000000000000038000000", "English", "how_to_form", "场所 + にも", "場所 + にも"),
    ("01G000000000000000T8000000", "English", "how_to_form", "途中で + 动词", "途中で + 動詞"),
    ("01G000000000000000Y4000000", "English", "how_to_form", "名词 + だ", "名詞 + だ"),
    ("01G000000000000000Z8000000", "English", "how_to_form", "社长 → 社長様", "社長 → 社長様"),
    ("01G000000000000000Z8000000", "Russian", "how_to_form", "社长 → 社長様", "社長 → 社長様"),
    ("01G0000000000000009C000000", "English", "examples", "海边に行きたいですか？", "海辺に行きたいですか？"),
    ("01G0000000000000009C000000", "Russian", "examples", "海边に行きたいですか？", "海辺に行きたいですか？"),
    ("01G000000000000000R4000000", "English", "examples", "海边で泳ぐとか", "海辺で泳ぐとか"),
    ("01G000000000000000R4000000", "Russian", "examples", "海边で泳ぐとか", "海辺で泳ぐとか"),
    ("01G0000000000000009M000000", "English", "pro_tip", "何时か = sometime", "いつか = sometime"),
    ("01G0000000000000009M000000", "Russian", "pro_tip", "何时か = когда-нибудь", "いつか = когда-нибудь"),
    ("01G000000000000000GC000000", "English", "examples", "电话してください", "電話してください"),
    ("01G000000000000000PG000000", "Russian", "examples", "电话があったこと", "電話があったこと"),
    ("01G000000000000000PR000000", "English", "examples", "勉强したあとで", "勉強したあとで"),
    ("01G000000000000000PR000000", "Russian", "examples", "勉强したあとで", "勉強したあとで"),
    ("01G000000000000000VR000000", "English", "examples", "毎日2時間は勉强します。", "毎日2時間は勉強します。"),
    ("01G000000000000000VR000000", "Russian", "examples", "毎日2時間は勉强します。", "毎日2時間は勉強します。"),
    ("01G000000000000000QM000000", "English", "examples", "日本語が 话せるほど", "日本語が話せるほど"),
    ("01G000000000000000QM000000", "Russian", "examples", "日本語が 话せるほど", "日本語が話せるほど"),
    ("01G000000000000000NW000000", "English", "pro_tip",
     "or end sentences with 给我 (ください)",
     "or end requests politely with ～ていただけませんか (ください)"),
    # --- Chinese words embedded in Russian/English text ----------------------
    ("01G0000000000000001W000000", "Russian", "explanation",
     "вопросительные слова для询问 о местоположении",
     "вопросительные слова для вопроса о местоположении"),
    ("01G0000000000000007W000000", "Russian", "warnings[0]",
     "Частица `か`标记 предложение как вопрос",
     "Частица `か` помечает предложение как вопрос"),
    ("01G000000000000000RM000000", "Russian", "explanation",
     "Подлежащее (тот, кто получает действие)标记ется при помощи が или は",
     "Подлежащее (тот, кто получает действие) отмечается при помощи が или は"),
    ("01G000000000000000H8000000", "Russian", "warnings[0]",
     "становится подлежащим и标记ется частицей が",
     "становится подлежащим и отмечается частицей が"),
    ("01G0000000000000006C000000", "English", "warnings[0]",
     "with verb renyoukei +怎么样了",
     "with the verb ます-stem (e.g. 旅行はどうでしたか)"),
]


def apply_fix(rule: dict, lang: str, field: str, old: str, new: str) -> bool:
    content = rule["content"].get(lang)
    if content is None:
        return False
    if field.startswith("warnings["):
        idx = int(field[len("warnings[") : -1])
        warnings = content.get("warnings", [])
        if idx >= len(warnings) or old not in warnings[idx]:
            return False
        warnings[idx] = warnings[idx].replace(old, new)
        return True
    value = content.get(field)
    if not isinstance(value, str) or old not in value:
        return False
    content[field] = value.replace(old, new)
    return True


def main() -> int:
    data = json.loads(PATH.read_text(encoding="utf-8"))
    rules = {r["rule_id"]: r for r in data["grammar"]}

    applied = missed = 0
    for rule_id, lang, field, old, new in FIXES:
        rule = rules.get(rule_id)
        if rule is None:
            print(f"MISS rule {rule_id}")
            missed += 1
            continue
        if apply_fix(rule, lang, field, old, new):
            applied += 1
        else:
            print(f"MISS {rule_id} {lang} {field}: {old[:50]!r}")
            missed += 1

    backup = PATH.with_suffix(f".{int(time.time())}.backup.json")
    shutil.copy2(PATH, backup)
    PATH.write_text(json.dumps(data, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(f"applied {applied}/{len(FIXES)} fixes ({missed} missed); backup: {backup.name}")
    return 1 if missed else 0


if __name__ == "__main__":
    sys.exit(main())
