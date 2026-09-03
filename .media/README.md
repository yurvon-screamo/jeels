# Медиа-ассеты Origa

Карта медиа-файлов репозитория: где что лежит, кто куда ссылается, правила для новых файлов.

## Зоны репозитория

| Путь | Содержимое | В git | Кто использует |
| --- | --- | --- | --- |
| `.media/` | Скрины приложений, стор-ассеты, баннеры, бренд-графика | да | `docs/landing-content-plan.md` (указатель на карту); стор- и маркетинг-материалы по конвенциям ниже |
| `marketing/` | Стратегии, исследования, блог-черновики, визитка `ru.page.png`, арт маскота в `draft_img/` | да | `docs/decisions/*`, `.qlty/qlty.toml` (secrets-scan excludes) |
| `origa_landing/public/` | Деплой-ассеты лендинга: page images, og-image, favicon | да | `origa_landing/src` (`home.rs`, `features.rs`, `seo.rs`, `integrations.rs`), корневые README |
| `origa_ui/public/` | Лого приложения (1024/128/32) + `external_icons/` — лого конкурентов | да | `origa_ui` (лого); favicon лендинга — байт-в-байт копия `logo-32.png` |
| `tauri/icons/`, `tauri/gen/**`, `tauri/msix/Assets/` | Иконки приложений — генерённые | да | сборка Tauri; руками не править |
| `origa/src/ocr/ocr_example.jpg` | Фикстура OCR-теста (1280×854) | да | `origa/src/ocr/tests.rs` |
| `cdn/` | Контент для CDN-деплоя: kanji SVG (`kanji_frames/`, `kanji_animations/`), шрифты, словари, грамматика, фразы, pitch, OCR-модель (`ndlocr/`), STT-модель (`whisper/`), артефакты updater (`releases/`), `well_known_set/` | **нет** (правило `cdn` в `.gitignore`) | `scripts/deploy_cdn.py`; `build.rs` берёт шрифты из `cdn/fonts/` |

Картинки интерфейса и лендинга между зонами не дублируются. Лендинг-изображения живут только в `origa_landing/public/images/`.

## .media/

### screenshots/

Десктоп (PNG 2880×1800):

- `screenshots/macos/en/` — grammar, home, lesson, lesson_answer, phrase, phrase_lesson, reading, writing
- `screenshots/macos/ru/` — answer, grammar, home, lesson, phrase, phrase_lesson, reading, writing
- `screenshots/windows/en/` — grammar, home, lesson, lesson_grammar, phrases, phrases_lesson, reading, writing
- `screenshots/windows/ru/` — grammar, home, lesson, lesson_phrase, phrase, reading, writing

Наборы экранов en/ru различаются (скрины снимались в разное время). При обновлении ориентируйся на текущий UI, а не на списки выше.

Мобильные (`screenshots/mobile/`, legacy-нейминг `{lang}.{device}.{screen}`):

- phone: `ru.phone.{main,grammar,lesson_kanji,phrases,sets}.png` (693×1502)
- tablet: `en.tablet.{main,card_set,grammar,phrases}.jpg`, `ru.tablet.{home,card_set,grammar,phrases}.jpg`
- `trim-16-9/` — обрезка 16:9 пяти phone-скринов для галерей и видео
- `screenshots/ipad/` — home, onboarding (legacy: нейминг без `{lang}`/`{device}`, вне конвенции)

### store/

Ассеты Google Play, 7 скринов на язык (PNG 1024×1536):

- `store/ru/` и `store/en/` — `1_hero`, `2_writing`, `3_quiz_audio`, `4_lesson`, `5_grammar`, `6_kanji_reading`, `7_phrase`
- `store/device/` (ru) и `store/device_en/` (en) — те же экраны в рамке устройства, 1284×2778
- `store/poster-1080x1080.png`, `store/poster-1440x2160.png` — постеры

Feature graphics (PNG 1024×500, требование Play): `banners/feature_graphics/feature-graphic-{en,ru}-{all-in-one,your-content}.png`.

### banners/ и бренд

- `banners/{en,ru}.all_in_one.png`, `banners/{en,ru}.your_content.png` — широкие баннеры (~1800×870, у каждого файла размер чуть отличается), исходники лендинг-коллажей
- `banners/cleaned_logo.png` — лого на чистом фоне (1024×1024)

### promo/ — нейро-сгенерированные промо-сеты

Сгенерированы через grok CLI (`image_gen`/`image_edit`), 2026-09, промпт-конвенция и клиент — см. wiki `experience/cli/grok-cli-image-generation.md` и `~/bin/grok-ask.py`.

| Каталог | Назначение | Язык | Состав |
| --- | --- | --- | --- |
| `promo/mobile-{ru,en}/` | RuStore, Reddit, X — 9:16 баннеры | RU / EN | 01-main, 02-phrases, 03-grammar, 04-kanji, 05-sets, 06-hero; `rustore/` — @1080x1920 |
| `promo/ph-{en,ru}/` | Product Hunt gallery + универсальная презентация | EN / RU | 01-hero, 02-home, 03-kanji, 04-grammar, 05-phrases, 06-privacy; `producthunt/` — @1270x760 |
| `promo/apple-iphone-{en,ru}/` | App Store iPhone (нативный ~9:19.5) | EN / RU | 01-main, 02-phrases, 03-grammar, 04-kanji, 05-sets; `appstore/` — @1320x2868 |
| `promo/apple-ipad-{en,ru}/` | App Store iPad | EN / RU | 01-main, 02-phrases, 03-grammar; `appstore/` — @2064x2752 (3:4 cover-crop) |
| `promo/apple-macos-{en,ru}/` | Mac App Store | EN / RU | 01-home, 02-lesson, 03-reading; `appstore/` — @1280x800 |
| `promo/social/` | X/Reddit: 16:9 баннеры и 1:1 посты | EN / RU | `16x9-{en,ru}`, `1x1-{en,ru}`; `x-twitter/` — @1200x675, @1080x1080 |
| `promo/store-assets/` | Иконки площадок | — | `icon-rustore-512.png`, `icon-producthunt-240.png` (из `origa_ui/public/logo.png`) |

Подкаталоги `appstore/`, `producthunt/`, `rustore/`, `x-twitter/` — производные под точные требования площадок (cover-crop/ресайз Lanczos из оригиналов выше). Оригиналы не удалять — источник производных.

Правила серии: единый стиль-бриф (крем-бумага #F7F2E7, тетрадная сетка, тёмно-зелёный сэриф, сакура); юзернейм на скринах заменён на `tanaka`; UI в EN-версиях нейро-переведён. QA: 3 раунда — большие тексты вычитать перед использованием, мелкий кегль может содержать артефакты. Известные анти-паттерны генерации (не повторять): упоминание «App Store» в промпте рисует страницу стора внутри UI; два референса конфликтуют; letter-by-letter спеллинги не работают.

## Конвенции

1. Новые десктоп-скрины: `.media/screenshots/{platform}/{lang}/{screen}.png`, platform — `macos|windows|linux`, lang — `en|ru`, screen — snake_case, один экран — один файл.
2. Новые стор-скрины: `.media/store/{lang}/N_topic.png` (1024×1536); device-версии — в `store/device/` (ru) и `store/device_en/` (en), 1284×2778.
3. Лендинг-изображения создаются только в `origa_landing/public/images/` (`{lang}.{name}.png`), копии в `.media/` не делаются.
4. Корневые `README.md`, `README.ru.md` и `marketing/product-hunt.md` ссылаются на `origa_landing/public/images/` относительными путями из корня репо. Переименовывая или удаляя файлы там, обновляй корневые README в том же PR.
5. `tauri/icons*` и `tauri/gen/**` — генерённые (tauri icon, platform gen). Ручная правка теряется при следующей регенерации; источник — исходник лого.
6. `origa_ui/public/external_icons/` — товарные знаки сторонних продуктов (Anki, Duolingo, Irodori, Migii, Minna no Nihongo) для страницы сравнения. Вне этого контекста не использовать.

## Provenance

- 2026-09: из корня `.media/` удалены 10 md5-дублей лендинг-картинок (канон — `origa_landing/public/images/`); удалён `.media/content.png` — разошёлся с landing-версией при одинаковых размерах, ссылок не имел (старый вариант — в git-истории до этого коммита); из корня репо удалён `dns-page.png` — скрин Cloudflare 404, артефакт отладки #372.
- `.gitignore` содержит правило `.media/store/.fonts/` — каталога нет, правило защитное: рабочие шрифты стора не должны попадать в коммиты.
