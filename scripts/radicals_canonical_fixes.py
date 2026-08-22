# ruff: noqa: E501
"""Canonical corrections for mislabelled radicals (RU + EN together).

The original radicals.json was authored with several non-canonical Russian
labels (e.g. 虍 = «Кошка» instead of «Тигр», 厂/匚 = «Крыша», 巛/阡 = «Гора»,
无 = «Бесконечность» — the opposite of its meaning). This table carries the
canonical Kangxi-radical meanings, grounded in the component's actual usage
(the kanji lists in radicals.json) where the component is a pure shape
(マ/ユ/ヨ/彑).

Each tuple: (char, name_ru, desc_ru, name_en, desc_en). Applied by
apply_radicals_fixes.py AFTER apply_radicals_en.py (the initial EN pass);
run order matters only on a pristine file.
"""

CORRECTIONS = [
    ("ノ", "Наклонная черта", "Наклонная черта, графический вариант радикала 丿; используется как элемент формы в верхних и левых частях иероглифов.", "Slash", "A slanted stroke, the graphical form of the 丿 radical; used as a shape element in the top and left parts of kanji."),
    ("亅", "Крючок", "Вертикальная черта, оканчивающаяся крючком; графический элемент в нижней части иероглифов.", "Hook", "A vertical stroke ending in a hook; a shape element at the bottom of kanji."),
    ("亠", "Крышка", "Изображает крышку или покрытие сверху; встречается в верхней части иероглифов (京, 交).", "Lid", "Depicts a lid or a cover on top; appears in the upper part of kanji (京, 交)."),
    ("儿", "Ноги", "Изображает ноги человека; встречается в нижней части иероглифов, связанных с людьми и их положением (兄, 元, 光).", "Legs", "Depicts a person's legs; appears at the bottom of kanji related to people and their stance (兄, 元, 光)."),
    ("ハ", "Деление", "Графический вариант цифры 八 — «восемь, деление»; изображает разделение на две части.", "Divide", "A graphical variant of 八 'eight'; depicts a split into two parts."),
    ("冂", "Рамка", "Рамка, открытая снизу; обозначает границу и охват территории (同, 円).", "Border box", "A box open at the bottom; denotes a border or an enclosure (同, 円)."),
    ("勹", "Обёртка", "Изображает охватывающую руку или обёртку; компонент-«обнималка» в знаках вроде 包, 句.", "Wrap", "Depicts a wrapping arm; the enveloping component in kanji like 包, 句."),
    ("匕", "Ложка", "Изображает ложку; встречается в знаках сравнения и указания (比, 北).", "Spoon", "Depicts a spoon; appears in kanji of comparison and direction (比, 北)."),
    ("匚", "Коробка", "Изображает ящик, открытый справа; относится к вместилищам и их содержимому (医, 匠, 匹).", "Box", "Depicts a box open on the right; relates to containers and their contents (医, 匠, 匹)."),
    ("卩", "Печать", "Изображает печать или сидящую фигуру; компонент в знаках, связанных с печатями и статусом (印, 卵).", "Seal", "Depicts a seal or a seated figure; a component in kanji related to seals and status (印, 卵)."),
    ("厂", "Утёс", "Изображает нависающий обрыв или склон; встречается в знаках, связанных со скалами и рельефом (原, 厚).", "Cliff", "Depicts an overhanging cliff or slope; appears in kanji related to rocks and terrain (原, 厚)."),
    ("凵", "Сосуд", "Изображает открытый сверху сосуд; используется, когда что-то помещено внутрь (凶, 函).", "Receptacle", "Depicts a container open at the top; used when something is placed inside (凶, 函)."),
    ("マ", "Форма マ", "Графический компонент в форме катаканы マ (изогнутая черта); верхняя часть знаков вроде 柔, 矛, 勇, 通.", "Ma shape", "A graphical component shaped like katakana マ (an angled stroke); the top part of kanji like 柔, 矛, 勇, 通."),
    ("ユ", "Форма ユ", "Графический компонент в форме катаканы ユ; верхняя часть знаков вроде 為, 快, 決.", "Yu shape", "A graphical component shaped like katakana ユ; the top part of kanji like 為, 快, 決."),
    ("乃", "Именно", "Классическое слово со значением «именно, тогда»; компонент в знаках вроде 孕, 仍, 秀.", "Namely", "A classical word meaning 'namely, thereupon'; a component in kanji like 孕, 仍, 秀."),
    ("尚", "Почитание", "Значение «по-прежнему, почитать»; верхняя часть знаков вроде 常, 堂, 裳.", "Esteem", "Means 'still, to esteem'; the top part of kanji like 常, 堂, 裳."),
    ("夂", "Медленный шаг", "Изображает идущую фигуру или след; значение «идти медленно, следовать»; японское имя радикала — ふゆがしら («зимняя голова»).", "Slow step", "Depicts a walking figure or a footprint; means 'to go slowly, to follow'; the radical's Japanese name is ふゆがしら ('winter head')."),
    ("彳", "Идущий", "Изображает идущего человека; левая половина знака 行; относится к движению и дорогам (往, 徒, 待).", "Going man", "Depicts a walking person; the left half of 行; relates to movement and roads (往, 徒, 待)."),
    ("攵", "Удар", "Изображает руку с палкой, наносящую удар; значение «ударять»; японское имя — ぼくづくり (教, 数, 敗).", "Tap", "Depicts a hand holding a stick, striking; means 'to tap, to strike'; the Japanese radical name is ぼくづくり (教, 数, 敗)."),
    ("囗", "Ограда", "Изображает ограду или огороженное пространство; охват и границы (国, 団, 困).", "Enclosure", "Depicts a fence or an enclosed space; enclosure and bounds (国, 団, 困)."),
    ("黹", "Вышивка", "Изображает вышивку или стежки; относится к шитью и рукоделию.", "Embroidery", "Depicts embroidery or stitches; relates to sewing and needlework."),
    ("尢", "Согнутый", "Изображает согнутенную фигуру; относится к значениям изгиба, хромоты и отклонения.", "Lame", "Depicts a bent figure; relates to crookedness, lameness, and deviation."),
    ("屮", "Росток", "Изображает прорастающий побег травы; графический вариант знака 艸.", "Sprout", "Depicts a sprouting grass shoot; a graphical variant of 艸."),
    ("巛", "Извилистая река", "Графический вариант радикала 川; изображает извилистый поток воды.", "Winding river", "A graphical variant of 川; depicts a winding stream of water."),
    ("已", "Уже", "Значение «уже»; один из трёх сходных знаков 己, 已, 巳.", "Already", "Means 'already'; one of the three similar characters 己, 已, 巳."),
    ("干", "Сухой", "Значение «сухой, сохнуть»; исторически изображает щит; компонент в знаках вроде 刊, 幹, 汗.", "Dry", "Means 'dry'; historically depicted a shield; a component in kanji like 刊, 幹, 汗."),
    ("廴", "Шаг", "Изображает длинный шаг вперёд; относится к движению и продвижению (建, 延).", "Stride", "Depicts a long step forward; relates to movement and progress (建, 延)."),
    ("廾", "Две руки", "Изображает две поднятые руки; графически совпадает с числом двадцать; компонент действий двумя руками (開, 弄).", "Two hands", "Depicts two raised hands; graphically identical to the number twenty; a component of two-handed actions (開, 弄)."),
    ("止", "Стоп", "Изображает стопу, стоящую на земле; значение «останавливаться» (止まる); компонент остановки и неподвижности (歩, 此).", "Stop", "Depicts a foot planted on the ground; means 'to stop' (止まる); a component of halting and stillness (歩, 此)."),
    ("亡", "Гибель", "Значение «погибать, исчезать, терять»; относится к утрате и исчезновению (亡くす, 忙, 忘).", "Perish", "Means 'to perish, to disappear, to lose'; relates to loss and disappearance (亡くす, 忙, 忘)."),
    ("ヨ", "Форма ヨ", "Графический вариант знака 彐 в форме катаканы ヨ; встречается в знаках вроде 君, 帰, 兼.", "Yo shape", "A graphical variant of 彐 shaped like katakana ヨ; appears in kanji like 君, 帰, 兼."),
    ("彑", "Пята", "Редкий вариант радикала 彐 — «пята»; верхняя часть знаков вроде 互, 彙.", "Snout (variant)", "A rare variant of the 彐 radical 'snout'; the top part of kanji like 互, 彙."),
    ("彡", "Узор", "Три штриха, изображающие волосы, перья или декоративный узор; передаёт идею украшения (形, 彩).", "Pattern", "Three strokes depicting hair, feathers, or a decorative pattern; conveys ornamentation (形, 彩)."),
    ("阡", "Межа", "Полевая тропа или межа между рисовыми полями; относится к дорогам и границам полей.", "Footpath", "A footpath or a boundary between rice fields; relates to field paths and borders."),
    ("无", "Ничто", "Вариант знака 無; значение «нет, ничто», отрицание наличия.", "Nothing", "A variant of 無; means 'nothing, not have', the negation of existence."),
    ("曰", "Говорить", "Изображает слово, исходящее изо рта; значение «говорить, изрекать».", "Say", "Depicts a word coming out of a mouth; means 'to say, to utter'."),
    ("歹", "Смерть", "Изображает обнажённую кость; относится к смерти, гибели и дурному (死, 残, 危).", "Death", "Depicts a bare bone; relates to death, demise, and the bad (死, 残, 危)."),
    ("爻", "Черты гексаграммы", "Скрещенные линии гексаграмм Ицзина; символ гадания и перемен (卦).", "Hexagram lines", "Crossing lines of the I Ching hexagrams; a symbol of divination and change (卦)."),
    ("爿", "Полено", "Зеркальная форма дерева 木; изображает расколотое бревно, левую половину дерева (状, 壮).", "Split wood", "The mirror form of 木; depicts a split log, the left half of a tree (状, 壮)."),
    ("片", "Половина", "Правая половина дерева, пара к 爿; значения части, половины и фрагмента (版, 牌).", "Half", "The right half of a tree, the counterpart of 爿; denotes a part, a half, a fragment (版, 牌)."),
    ("巴", "Томоэ", "Изогнутый элемент-запятая (томоэ); по происхождению — змея или коготь; компонент в знаках вроде 把, 色, 肥.", "Tomoe", "A curved comma-shaped element (tomoe); originally a serpent or a claw; a component in kanji like 把, 色, 肥."),
    ("疋", "Кусок ткани", "Изображает свёрнутую ткань; мера ткани (рулон); графически часто дублирует 足 (楚, 疎).", "Bolt of cloth", "Depicts rolled cloth; a cloth measure (a bolt); graphically often doubles for 足 (楚, 疎)."),
    ("癶", "Шаги врозь", "Две стопы, направленные в противоположные стороны; движение и попеременный шаг (登, 発).", "Footsteps apart", "Two feet pointing in opposite directions; movement and alternating steps (登, 発)."),
    ("禹", "Император Юй", "Имя легендарного императора Юя, укротившего потоп; компонент в знаках вроде 属, 遇, 愚.", "Emperor Yu", "The name of the legendary Emperor Yu who tamed the flood; a component in kanji like 属, 遇, 愚."),
    ("立", "Стоять", "Изображает человека, стоящего на земле; значение «стоять» (位).", "Stand", "Depicts a person standing on the ground; means 'to stand' (位)."),
    ("羊", "Овца", "Указывает на овцу, козу или связанные с ними понятия, такие как шерсть и пастбище.", "Sheep", "Points to a sheep, a goat, or related concepts such as wool and pasture."),
    ("艮", "Остановка", "Значение «тупой, останавливающийся»; в Ицзине — триграмма «гора»; компонент в знаках вроде 良, 限, 根.", "Stopping", "Means 'blunt, coming to a stop'; the 'mountain' trigram of the I Ching; a component in kanji like 良, 限, 根."),
    ("虍", "Тигр", "Изображает шкуру тигра; верхняя часть знака 虎 и родственных ему (虚, 慮).", "Tiger", "Depicts a tiger's hide; the top part of 虎 and its kin (虚, 慮)."),
    ("豸", "Барсук", "Изображает барсука — мелкого хищного зверя; относится к зверям и их повадкам (豹, 貌).", "Badger", "Depicts a badger, a small predatory animal; relates to beasts and their traits (豹, 貌)."),
    ("辰", "Знак дракона", "Значение «утро, небесное тело»; пятый земной знак зодиака — дракон; компонент в знаках вроде 農, 振.", "Zodiac dragon", "Means 'morning, celestial body'; the fifth zodiac sign — the dragon; a component in kanji like 農, 振."),
    ("酉", "Сосуд для сакэ", "Изображает глиняный сосуд для сакэ; японское имя радикала — とり («птица», знак петуха в зодиаке); знаки алкоголя (酒, 酔).", "Sake vessel", "Depicts a clay sake jar; its Japanese radical name is とり ('bird', the zodiac rooster); marks alcohol kanji (酒, 酔)."),
    ("釆", "Зёрна риса", "Изображает рассыпанные зёрна; передаёт значение различения (番, 悉, 釈).", "Grain", "Depicts scattered grains; conveys the meaning of distinguishing (番, 悉, 釈)."),
    ("舛", "Противоречие", "Стопы, повёрнутые в противоположные стороны; значения разлада и ошибки (舞).", "Discord", "Feet turned in opposite directions; the meanings of discord and error (舞)."),
    ("隶", "Подчинение", "Изображает руку, хватающую добычу за хвост; значения «подчинять, раб» (逮, 隷, 康).", "Subjugate", "Depicts a hand grabbing prey by the tail; the meanings of subduing and servitude (逮, 隷, 康)."),
    ("奄", "Покрытие", "Значение «покрывать, внезапно»; компонент в знаках вроде 掩, 俺, 庵.", "Cover", "Means 'to cover, suddenly'; a component in kanji like 掩, 俺, 庵."),
    ("飛", "Полёт", "Изображает летящую птицу; значение «лететь»; полёт и быстрота.", "Fly", "Depicts a bird in flight; means 'to fly'; flight and swiftness."),
    ("韋", "Выделанная кожа", "Значение «дублёная кожа»; в древних знаках — выделка кожи и оборона (衛).", "Tanned leather", "Means 'tanned leather'; in old kanji — leatherwork and defense (衛)."),
    ("齊", "Равный (трад.)", "Традиционная форма знака 斉; значение «равный, стройный, согласовать».", "Equal (traditional)", "The traditional form of 斉; means 'equal, neat, to align'."),
]
