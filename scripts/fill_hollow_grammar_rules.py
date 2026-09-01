#!/usr/bin/env python3
"""Phase 3b content pass: fill the hollow conjugation rules in grammar_v2.

Seven script-generated rules (honorific imperatives + i-adjective forms)
had only a title/short description. This pass adds learner-facing
explanation / how_to_form / examples / structured nuances / pro_tip in
both languages, plus search keywords. The three honorific entries also
had a dev-facing FormatAction stub as explanation — replaced with proper
teaching text.

Run: python scripts/fill_hollow_grammar_rules.py
"""

from __future__ import annotations

import json
import shutil
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PATH = ROOT / "cdn" / "grammar" / "grammar_v2.json"

NASAI = {
    "rule_id": "01KXAX0VRPNSR58CP87HJPXYMM",
    "keywords": [["なさる", "なさい"]],
    "English": {
        "explanation": "`なさい` is the polite imperative of the honorific verb `なさる` (to do — respectful). It turns a verb's ます-stem into a firm but respectful instruction, softer than the blunt `～ろ` imperative and common from teachers, doctors and parents: `食べなさい` — \"eat (please)\".",
        "how_to_form": "| Element | Form | Example |\n|---------|------|---------|\n| Verb ます-stem | stem + なさい | 食べる → 食べなさい |\n| する (exception) | し + なさい | する → しなさい |\n| 来る (exception) | 来 + なさい | 来る → 来なさい |",
        "examples": "```\n宿題をしなさい。\nDo your homework.\n```\n\n```\nもう寝なさい。\nGo to bed now.\n```\n\n```\nお薬を飲みなさい。\nTake your medicine.\n```",
        "nuances": {
            "common_mistakes": [
                {"wrong": "Using なさい with the dictionary form (食べるなさい)", "correct": "Attach なさい to the ます-stem: 食べなさい", "note": None},
                {"wrong": "Using なさい with strangers or superiors", "correct": "It sounds like a parent/teacher; with outsiders use ～てください", "note": None},
            ],
            "notes": [{"tag": "register", "text": "なさい is respectful in origin but sounds top-down: fine from a doctor, odd from a junior employee."}],
        },
        "pro_tip": "なさい keeps the feel of a warm instruction rather than an order — think of how a parent says 「食べなさい」 at dinner.",
    },
    "Russian": {
        "explanation": "`なさい` — вежливый императив уважительного глагола `なさる` («делать» в почтительной форме). Он присоединяется к основе ます-формы глагола и образует настойчивую, но уважительную инструкцию — мягче грубого императива `～ろ`. Типично для учителей, врачей и родителей: `食べなさい` — «ешь (пожалуйста)».",
        "how_to_form": "| Элемент | Форма | Пример |\n|---------|------|---------|\n| Основа ます-формы | основа + なさい | 食べる → 食べなさい |\n| する (исключение) | し + なさい | する → しなさい |\n| 来る (исключение) | 来 + なさい | 来る → 来なさい |",
        "examples": "```\n宿題をしなさい。\nСделай домашнее задание.\n```\n\n```\nもう寝なさい。\nА теперь спать.\n```\n\n```\nお薬を飲みなさい。\nПрими лекарство.\n```",
        "nuances": {
            "common_mistakes": [
                {"wrong": "Присоединять なさい к словарной форме (食べるなさい)", "correct": "なさい присоединяется к основе ます-формы: 食べなさい", "note": None},
                {"wrong": "Использовать なさい с незнакомыми или старшими", "correct": "Форма звучит свысока; с посторонними используйте ～てください", "note": None},
            ],
            "notes": [{"tag": "register", "text": "なさい уважителен по происхождению, но звучит по-родительски: нормально от врача, странно от младшего коллеги."}],
        },
        "pro_tip": "なさい — это тёплая инструкция, а не приказ: как родитель говорит за столом 「食べなさい」.",
    },
}

KUDASAI = {
    "rule_id": "01KXAX0VRPXYAP6KYN9P7CJTPA",
    "keywords": [["くださる", "ください"]],
    "English": {
        "explanation": "`ください` is the special imperative of the honorific verb `くださる` (to give — respectful). Attached to the て-form it makes the standard polite request `～てください`, and after a noun (+ を) it asks for a thing: `名前を書いてください` — \"please write your name\".",
        "how_to_form": "| Element | Form | Example |\n|---------|------|---------|\n| Verb て-form | て + ください | 書く → 書いてください |\n| Noun | Noun + を + ください | 水 + を + ください |",
        "examples": "```\nもう一度言ってください。\nPlease say it once more.\n```\n\n```\nここに名前を書いてください。\nPlease write your name here.\n```\n\n```\n水をください。\nWater, please.\n```",
        "nuances": {
            "common_mistakes": [
                {"wrong": "Requesting an action with just a noun (お願いしますください)", "correct": "Nouns take を + ください; actions take て-form + ください", "note": None},
                {"wrong": "Stemming ください from the plain verb くれる", "correct": "It is the imperative of the honorific くださる — that is what makes the request polite", "note": None},
            ],
            "notes": [{"tag": "register", "text": "～てください is polite but still a direct request; even softer is ～ていただけませんか."}],
        },
        "pro_tip": "Think of ください as the respectful twin of くれる: same giving idea, raised one politeness level.",
    },
    "Russian": {
        "explanation": "`ください` — особый императив уважительного глагола `くださる` («давать» в почтительной форме). С て-формой он образует стандартную вежливую просьбу `～てください`, а после существительного (с を) просит предмет: `名前を書いてください` — «напишите, пожалуйста, ваше имя».",
        "how_to_form": "| Элемент | Форма | Пример |\n|---------|------|---------|\n| て-форма глагола | て + ください | 書く → 書いてください |\n| Существительное | Сущ. + を + ください | 水 + を + ください |",
        "examples": "```\nもう一度言ってください。\nСкажите, пожалуйста, ещё раз.\n```\n\n```\nここに名前を書いてください。\nНапишите, пожалуйста, имя здесь.\n```\n\n```\n水をください。\nВоды, пожалуйста.\n```",
        "nuances": {
            "common_mistakes": [
                {"wrong": "Просить действие через существительное (お願いしますください)", "correct": "Существительное + を + ください; действие — て-форма + ください", "note": None},
                {"wrong": "Считать, что ください происходит от обычного くれる", "correct": "Это императив уважительного くださる — именно он даёт вежливость", "note": None},
            ],
            "notes": [{"tag": "register", "text": "～てください вежливо, но всё же прямая просьба; мягче — ～ていただけませんか."}],
        },
        "pro_tip": "ください — почтительный близнец くれる: та же идея «давания», но на уровень вежливее.",
    },
}

IRASSHAI = {
    "rule_id": "01KXAX0VRP192A16Z4NJ2WK9GA",
    "keywords": [["いらっしゃる", "いらっしゃい"]],
    "English": {
        "explanation": "`いらっしゃい` is the imperative of the honorific verb `いらっしゃる` (to be/to go/to come — respectful). As a greeting, `いらっしゃいませ` welcomes customers into a shop, while a plain `いらっしゃい` greets a guest at home — literally \"honored that you came\".",
        "how_to_form": "| Use | Form | Nuance |\n|-----|------|--------|\n| Shop greeting | いらっしゃいませ | Polite, service register |\n| Home greeting | いらっしゃい | Warm, less formal |",
        "examples": "```\nいらっしゃいませ。何名様ですか。\nWelcome! How many people?\n```\n\n```\nやあ、いらっしゃい。どうぞお上がりください。\nOh, welcome! Please come in.\n```\n\n```\n毎度いらっしゃいませ。\nThank you for shopping with us (again).\n```",
        "nuances": {
            "common_mistakes": [
                {"wrong": "Greeting a friend with いらっしゃいませ", "correct": "Use it for customers/guests; with friends a plain ようこそ or やあ fits", "note": None},
            ],
            "notes": [{"tag": "variation", "text": "いらっしゃる also covers honorific «to be» and «to go»: 社長がいらっしゃいました — the president is here."}],
        },
        "pro_tip": "The ませ ending is old-fashioned polite ます-imperative — you will hear it mostly in service Japanese.",
    },
    "Russian": {
        "explanation": "`いらっしゃい` — императив уважительного глагола `いらっしゃる` («быть/идти/приходить» в почтительной форме). Как приветствие `いらっしゃいませ` встречает покупателя в магазине, а простое `いらっしゃい` — гостя дома: буквально «почтён вашим приходом».",
        "how_to_form": "| Случай | Форма | Оттенок |\n|-----|------|---------|\n| Приветствие в магазине | いらっしゃいませ | Вежливо, сервисный регистр |\n| Приветствие дома | いらっしゃい | Тепло, менее формально |",
        "examples": "```\nいらっしゃいませ。何名様ですか。\nДобро пожаловать! Сколько вас человек?\n```\n\n```\nやあ、いらっしゃい。どうぞお上がりください。\nО, добро пожаловать! Проходите, пожалуйста.\n```\n\n```\n毎度いらっしゃいませ。\nСпасибо, что снова к нам заглянули.\n```",
        "nuances": {
            "common_mistakes": [
                {"wrong": "Приветствовать друга формой いらっしゃいませ", "correct": "Она — для клиентов и гостей; с друзьями — просто ようこそ или やあ", "note": None},
            ],
            "notes": [{"tag": "variation", "text": "いらっしゃる также означает почтительное «быть» и «идти»: 社長がいらっしゃいました — президент здесь."}],
        },
        "pro_tip": "Окончание ませ — старинный вежливый императив от ます; в живой речи он остался почти только в сервисном японском.",
    },
}

GARU = {
    "rule_id": "01KXB1CYH9BT21BNG51G47YJBH",
    "keywords": [["がる"]],
    "English": {
        "explanation": "Feelings are private in Japanese, so first-person adjectives like `嬉しい` (glad) become `～がる` when you describe someone else: `彼が嬉しがっている` — \"he looks glad\". Replace the final い with く and add がる (usually in the ている form for a current state).",
        "how_to_form": "| Element | Form | Example |\n|---------|------|---------|\n| i-adjective | stem (く) + がる | 嬉しい → 嬉しがる |\n| Current state | がっている | 嬉しがっている |\n| Third person past | がった | 寂しがった |",
        "examples": "```\n妹は新しい靴を欲しがっている。\nMy little sister wants new shoes.\n```\n\n```\n彼は怖がるから、犬を近づけないで。\nHe gets scared, so keep the dog away.\n```\n\n```\nみんなが行きたがっている。\nEveryone wants to go.\n```",
        "nuances": {
            "common_mistakes": [
                {"wrong": "彼は嬉しいです (direct feeling for a third person)", "correct": "彼は嬉しがっています — use がる for others' feelings", "note": None},
                {"wrong": "Dropping ている for a current state (彼が嬉しがる)", "correct": "嬉しがっている — ongoing emotional state", "note": None},
            ],
            "notes": [{"tag": "collocation", "text": "欲しい → 欲しがる and たい → たがる are the two most frequent がる patterns."}],
        },
        "pro_tip": "がる literally means \"to show signs of\" — you are reading behavior, not the person's heart.",
    },
    "Russian": {
        "explanation": "В японском чувства — личная территория: прилагательные от первого лица вроде `嬉しい` («рад») для третьего лица превращаются в `～がる`: `彼が嬉しがっている` — «он выглядит радостным». Замените конечное い на く и добавьте がる (обычно в форме ている для текущего состояния).",
        "how_to_form": "| Элемент | Форма | Пример |\n|---------|------|---------|\n| и-прилагательное | основа (く) + がる | 嬉しい → 嬉しがる |\n| Текущее состояние | がっている | 嬉しがっている |\n| Прошедшее 3-го лица | がった | 寂しがった |",
        "examples": "```\n妹は新しい靴を欲しがっている。\nМладшая сестра хочет новые туфли.\n```\n\n```\n彼は怖がるから、犬を近づけないで。\nОн пугается, так что не подпускай собаку.\n```\n\n```\nみんなが行きたがっている。\nВсе хотят пойти.\n```",
        "nuances": {
            "common_mistakes": [
                {"wrong": "彼は嬉しいです (прямое чувство о третьем лице)", "correct": "彼は嬉しがっています — для чужих чувств используйте がる", "note": None},
                {"wrong": "Пропуск ている для текущего состояния (彼が嬉しがる)", "correct": "嬉しがっている — длящееся эмоциональное состояние", "note": None},
            ],
            "notes": [{"tag": "collocation", "text": "欲しい → 欲しがる и たい → たがる — два самых частых がる-паттерна."}],
        },
        "pro_tip": "がる буквально значит «подавать признаки» — вы читаете поведение, а не сердце человека.",
    },
}

KUNAI = {
    "rule_id": "01KXB1CYH9ZDY4GMZW4FQVE7BH",
    "keywords": [["くない"]],
    "English": {
        "explanation": "To negate an i-adjective, replace the final い with くない: `高い` (expensive) → `高くない` (not expensive). The adjective stays conjugating like a verb from here on — past is くなかった, polite is くないです / くありません.",
        "how_to_form": "| Form | Rule | Example |\n|------|------|---------|\n| Plain negative | ～くない | 高い → 高くない |\n| Polite negative | ～くないです / ～くありません | 高くないです |\n| Before nouns | ～くない + Noun | 高くないレストラン |",
        "examples": "```\nこの本は高くないです。\nThis book is not expensive.\n```\n\n```\n今日は寒くない。\nIt is not cold today.\n```\n\n```\n彼の答えは正しくなかった。\nHis answer was not correct.\n```",
        "nuances": {
            "common_mistakes": [
                {"wrong": "Negating with です (高いですない)", "correct": "い-adjectives negate themselves: 高くないです", "note": None},
                {"wrong": "いい → いいくない", "correct": "いい is irregular: よくない", "note": None},
            ],
            "notes": [{"tag": "variation", "text": "くありません is the more formal written variant of くないです."}],
        },
        "pro_tip": "One rule of thumb: い-adjectives never need です to change form — です only adds politeness at the end.",
    },
    "Russian": {
        "explanation": "Отрицание и-прилагательного образуется заменой конечного い на くない: `高い` («дорогой») → `高くない` («недорогой»). Дальше прилагательное спрягается само: прошедшее — くなかった, вежливо — くないです / くありません.",
        "how_to_form": "| Форма | Правило | Пример |\n|------|------|---------|\n| Простое отрицание | ～くない | 高い → 高くない |\n| Вежливое отрицание | ～くないです / ～くありません | 高くないです |\n| Перед существительным | ～くない + Сущ. | 高くないレストラン |",
        "examples": "```\nこの本は高くないです。\nЭта книга недорогая.\n```\n\n```\n今日は寒くない。\nСегодня не холодно.\n```\n\n```\n彼の答えは正しくなかった。\nЕго ответ был неверным.\n```",
        "nuances": {
            "common_mistakes": [
                {"wrong": "Отрицать через です (高いですない)", "correct": "и-прилагательные сами образуют отрицание: 高くないです", "note": None},
                {"wrong": "いい → いいくない", "correct": "いい — исключение: よくない", "note": None},
            ],
            "notes": [{"tag": "variation", "text": "くありません — более формальный письменный вариант くないです."}],
        },
        "pro_tip": "Правило большого пальца: и-прилагательному не нужен です для изменения формы — です лишь добавляет вежливость в конце.",
    },
}

KUNAKATTA = {
    "rule_id": "01KXB1CYH9R2G7VEQ4TBFNJNQ5",
    "keywords": [["くなかった"]],
    "English": {
        "explanation": "The negative past of an i-adjective stacks both endings: く (negation) + なかった (past): `高い` → `高くなかった` — \"was not expensive\". Polite variants are くなかったです and くありませんでした.",
        "how_to_form": "| Form | Rule | Example |\n|------|------|---------|\n| Plain negative past | ～くなかった | 高い → 高くなかった |\n| Polite | ～くなかったです / ～くありませんでした | 高くなかったです |",
        "examples": "```\n昨日の試験は難しくなかった。\nYesterday's exam was not difficult.\n```\n\n```\n映画は面白くなかったです。\nThe movie was not interesting.\n```\n\n```\nおいしくなかったら、食べなくてもいいです。\nIf it does not taste good, you do not have to eat it.\n```",
        "nuances": {
            "common_mistakes": [
                {"wrong": "高くなかったですか mixing with 高かったですくない", "correct": "Order is fixed: negation first, past second — くなかった", "note": None},
                {"wrong": "いい → いくなかった", "correct": "いい is irregular: よくなかった", "note": None},
            ],
            "notes": [{"tag": "formality", "text": "くありませんでした is the formal written form; in speech くなかったです dominates."}],
        },
        "pro_tip": "The て-form of this pattern, ～くなくて, chains reasons: 忙しくなくて、映画を見た — \"wasn't busy, so I watched a movie\".",
    },
    "Russian": {
        "explanation": "Отрицательное прошедшее и-прилагательного совмещает оба показателя: く (отрицание) + なかった (прошедшее): `高い` → `高くなかった` — «не был дорогим». Вежливые варианты — くなかったです и くありませんでした.",
        "how_to_form": "| Форма | Правило | Пример |\n|------|------|---------|\n| Простое отрицательное прошедшее | ～くなかった | 高い → 高くなかった |\n| Вежливо | ～くなかったです / ～くありませんでした | 高くなかったです |",
        "examples": "```\n昨日の試験は難しくなかった。\nВчерашний экзамен был нетрудным.\n```\n\n```\n映画は面白くなかったです。\nФильм был неинтересным.\n```\n\n```\nおいしくなかったら、食べなくてもいいです。\nЕсли невкусно, можно не есть.\n```",
        "nuances": {
            "common_mistakes": [
                {"wrong": "Путать порядок показателей (高かったですくない)", "correct": "Порядок фиксирован: сначала отрицание, потом прошедшее — くなかった", "note": None},
                {"wrong": "いい → いくなかった", "correct": "いい — исключение: よくなかった", "note": None},
            ],
            "notes": [{"tag": "formality", "text": "くありませんでした — формальная письменная форма; в речи царит くなかったです."}],
        },
        "pro_tip": "て-форма этого паттерна, ～くなくて, сцепляет причины: 忙しくなくて、映画を見た — «был не занят, поэтому посмотрел фильм».",
    },
}

KEREBA = {
    "rule_id": "01KXB1CYH9NJNT9XRBR9ER06XN",
    "keywords": [["ければ"]],
    "English": {
        "explanation": "The conditional of an i-adjective replaces い with ければ: `安い` → `安ければ` — \"if it is cheap\". Use it for real conditions and general rules, just like the verb ば-form.",
        "how_to_form": "| Form | Rule | Example |\n|------|------|---------|\n| Plain conditional | ～ければ | 安い → 安ければ |\n| Negative conditional | ～くなければ | 高くなければ買う |",
        "examples": "```\n安ければ買います。\nIf it is cheap, I will buy it.\n```\n\n```\n天気が良ければ、出かけましょう。\nIf the weather is good, let's go out.\n```\n\n```\nおいしければ、また注文します。\nIf it tastes good, I will order it again.\n```",
        "nuances": {
            "common_mistakes": [
                {"wrong": "Using the たら-form of the adjective itself (安かったら is fine but different) where a bare condition is meant", "correct": "安ければ is the clean ば-conditional of the adjective", "note": None},
                {"wrong": "いい → いいければ", "correct": "いい is irregular: よければ", "note": None},
            ],
            "notes": [{"tag": "variation", "text": "For advice the set phrase ～なければなりません (\"must\") is built on the negative conditional of verbs and adjectives alike."}],
        },
        "pro_tip": "ければ is the adjective twin of the verb ば-form — same \"if A then B\" logic, zero surprise endings.",
    },
    "Russian": {
        "explanation": "Условная форма и-прилагательного заменяет い на ければ: `安い` → `安ければ` — «если дешёвый». Используется для реальных условий и общих правил — точно как ば-форма глагола.",
        "how_to_form": "| Форма | Правило | Пример |\n|------|------|---------|\n| Простое условие | ～ければ | 安い → 安ければ |\n| Отрицательное условие | ～くなければ | 高くなければ買う |",
        "examples": "```\n安ければ買います。\nЕсли будет дёшево, куплю.\n```\n\n```\n天気が良ければ、出かけましょう。\nЕсли погода будет хорошей, пойдём гулять.\n```\n\n```\nおいしければ、また注文します。\nЕсли будет вкусно, закажу ещё раз.\n```",
        "nuances": {
            "common_mistakes": [
                {"wrong": "Смешивать с たら-формой (安かったら — другой оттенок)", "correct": "安ければ — чистое ば-условие прилагательного", "note": None},
                {"wrong": "いい → いいければ", "correct": "いい — исключение: よければ", "note": None},
            ],
            "notes": [{"tag": "variation", "text": "На отрицательном условии построено устойчивое ～なければなりません («должен») — для глаголов и прилагательных одинаково."}],
        },
        "pro_tip": "ければ — прилагательный близнец глагольной ば-формы: та же логика «если A, то B», без сюрпризов.",
    },
}

FILLS = [NASAI, KUDASAI, IRASSHAI, GARU, KUNAI, KUNAKATTA, KEREBA]


def main() -> int:
    data = json.loads(PATH.read_text(encoding="utf-8"))
    rules = {r["rule_id"]: r for r in data["grammar"]}

    for fill in FILLS:
        rule = rules.get(fill["rule_id"])
        if rule is None:
            print(f"MISS rule {fill['rule_id']}")
            return 1
        if not rule.get("keywords"):
            rule["keywords"] = fill["keywords"]
        for lang in ("English", "Russian"):
            content = rule["content"][lang]
            for field in ("explanation", "how_to_form", "examples", "nuances", "pro_tip"):
                content[field] = fill[lang][field]

    backup = PATH.with_suffix(f".{int(time.time())}.backup.json")
    shutil.copy2(PATH, backup)
    PATH.write_text(json.dumps(data, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(f"filled {len(FILLS)} rules; backup: {backup.name}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
