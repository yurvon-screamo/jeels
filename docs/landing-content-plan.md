# SEO Landing Content Plan — Origa

> **Status:** Approved (post Round 2 review fixes)
> **Scope:** 6 pages × 2 languages = 12 localized pages
> **Prerequisite:** Pages MUST be served as SSR or static HTML. Current CSR architecture is NOT suitable.

---

## 1. Sitemap & URL Structure

### Routing Model

EN is default language at root. RU is prefixed with `/ru/`.

| Page | EN URL | RU URL |
| --- | --- | --- |
| Homepage | `/` | `/ru/` |
| Features | `/features` | `/ru/features` |
| Compare | `/compare` | `/ru/compare` |
| Download | `/download` | `/ru/download` |
| Privacy | `/privacy` | `/ru/privacy` |
| Terms | `/terms` | `/ru/terms` |

### Canonical & hreflang (every page)

```html
<!-- Example for EN homepage -->
<link rel="canonical" href="https://[DOMAIN]/" />
<link rel="alternate" hreflang="en" href="https://[DOMAIN]/" />
<link rel="alternate" hreflang="ru" href="https://[DOMAIN]/ru/" />
<link rel="alternate" hreflang="x-default" href="https://[DOMAIN]/" />
```

### SEO Infrastructure

- `robots.txt`: Allow all, reference sitemap.xml
- `sitemap.xml`: 12 URLs with lastmod + hreflang
- OG + Twitter Card tags on every page
- Semantic HTML (header, main, section, footer)

---

## 2. Navigation

Header: `[Logo] Origa    Features    Compare    Download`

Footer:

```text
Product           Legal              Resources
───────────         ──────────         ──────────
Features          Privacy Policy     GitHub
Compare           Terms of Service
Download
```

Language switch: EN | RU (in header or footer)

---

## 3. Meta Tags (all titles ≤50 chars, descriptions ≤125 chars)

### EN

| Page | Title | Description |
| --- | --- | --- |
| / | `Origa — All-in-One Japanese Learning App` | `All-in-one Japanese learning app: vocabulary, kanji, grammar, 200K+ native phrases and JLPT analytics. In your own language.` |
| /features | `Vocabulary, Kanji, Grammar & Listening \| Origa` | `Built-in dictionaries, smart flashcards, automatic furigana, JLPT grammar and 200K+ phrases with audio — all in one app.` |
| /compare | `Origa vs Anki, WaniKani, Bunpro & Duolingo` | `Compare Origa with Anki, WaniKani, Bunpro and Duolingo. See which Japanese learning tools work best together.` |
| /download | `Download Origa — Japanese Learning App` | `Download Origa for Windows, Linux, macOS or Android. All platforms support offline mode.` |
| /privacy | `Privacy Policy — Origa` | `Privacy policy for Origa Japanese learning application.` |
| /terms | `Terms of Service — Origa` | `Terms of service for Origa Japanese learning application.` |

### RU

| Page | Title | Description |
| --- | --- | --- |
| / | `Origa — Японский язык без посредников` | `Всё в одном приложении: лексика, кандзи, грамматика, 200 000+ фраз с озвучкой и аналитика JLPT. На вашем языке.` |
| /features | `Лексика, кандзи, грамматика и аудирование \| Origa` | `Встроенные словари, умные карточки, автоматическая фуригана, грамматика JLPT и 200 000+ фраз с аудио.` |
| /compare | `Origa vs Anki, WaniKani, Bunpro и Duolingo` | `Сравните Origa с Anki, WaniKani, Bunpro и Duolingo. Узнайте, какие инструменты лучше работают вместе.` |
| /download | `Скачать Origa — приложение для японского` | `Скачайте Origa для Windows, Linux, macOS или Android. Все платформы поддерживают офлайн-режим.` |
| /privacy | `Политика конфиденциальности — Origa` | `Политика конфиденциальности приложения Origa для изучения японского языка.` |
| /terms | `Условия использования — Origa` | `Условия использования приложения Origa для изучения японского языка.` |

### OG & Twitter Card (every page)

```text
og:title → from page title
og:description → from page description
og:image → 1200×630px, unique per page (or shared default)
og:url → canonical URL
og:type → website
og:locale → en_US or ru_RU
twitter:card → summary_large_image
twitter:title → from page title
twitter:description → from page description
twitter:image → same as og:image
```

---

## 4. Homepage (`/`) — Block Structure

### Block 1: Hero

**EN:**

- H1: `Learn Japanese in your own language`
- Subtitle: `Vocabulary, kanji, grammar, listening and 200,000+ native phrases — all in one app. No English required.`
- CTA primary: `Get started` → /download
- CTA secondary: `Open web app` → web app link

**RU:**

- H1: `Японский язык на вашем родном языке`
- Subtitle: `Лексика, кандзи, грамматика, аудирование и более 200 000 фраз с оригинальной озвучкой — в одном приложении. Без английского посредника.`
- CTA primary: `Начать` → /download
- CTA secondary: `Открыть в браузере` → web app link

### Block 2: Problem Statement

**EN H2:** `Learning Japanese shouldn't require learning English first`
**EN text:** `Most Japanese learning tools are built for English speakers. If your native language is Russian — with Vietnamese, Korean and Indonesian coming soon — you're stuck translating through a second language. Origa changes that. Every dictionary, every grammar explanation, every interface element — in your language from day one.`

**RU H2:** `Изучать японский не значит учить английский`
**RU text:** `Большинство инструментов для изучения японского созданы для англоговорящих. Если ваш родной язык — русский — а вьетнамский, корейский и индонезийский скоро появятся — вам приходится переводить через второй язык. Origa меняет это. Каждый словарь, каждое объяснение грамматики, каждый элемент интерфейса — на вашем языке с первого дня.`

### Block 3: Features Preview (4 cards → grid)

**EN H2:** `Everything you need. Nothing you don't.`
**RU H2:** `Всё, что нужно. Ничего лишнего.`

**Card 1 — Vocabulary:**

- EN: `Built-in dictionaries in your language. Create cards in seconds — type a word, scan a photo, or record audio.`
- RU: `Встроенные словари на вашем языке. Создавайте карточки за секунды — введите слово, отсканируйте фото или запишите аудио.`
- Link: `/features`

**Card 2 — Kanji:**

- EN: `Automatic furigana that hides as you learn. Writing practice with stroke order. JLPT-mapped kanji dictionaries.`
- RU: `Автоматическая фуригана, которая скрывается по мере изучения. Практика написания с правильным порядком черт.`
- Link: `/features`

**Card 3 — Grammar:**

- EN: `Structured grammar reference by JLPT level. Rules explained with words you already know. Practice tests.`
- RU: `Структурированный справочник грамматики по уровням JLPT. Правила объясняются на словах, которые вы уже знаете.`
- Link: `/features`

**Card 4 — 200K+ Phrases:**

- EN: `Phrases from anime, visual novels and native content with original voice acting. Auto-selected for your level.`
- RU: `Фразы из аниме, визуальных новелл и нативного контента с оригинальной озвучкой. Автоподбор по вашему уровню.`
- Link: `/features`

### Block 4: Core Principles

**EN H2:** `Built different`
**RU H2:** `Другой подход`

- ✦ `**Learn from your own content.** Study what you're actually reading, watching or listening to — not someone else's word list.`
- ✦ `**Smart spaced repetition.** FSRS algorithm adapts review intervals to your memory.`
- ✦ `**Everything runs locally.** All AI models and data processing happen on your device. Nothing is sent to the cloud.`
- ✦ `**Works offline.** Full functionality without internet. Study on the train, on a plane, anywhere.`

### Block 5: Platform Availability

**EN H2:** `Available everywhere`
**RU H2:** `Доступно везде`

Windows · Linux · macOS · Android · Web — all confirmed Ready.
`All platforms support offline mode.`

### Block 6: Compare Preview

**EN H2:** `Why choose Origa`
**RU H2:** `Почему Origa`

Condensed comparison table (key rows only):

- Features: Vocabulary, Kanji, Grammar, Listening
- Languages: Your language, English only, English only, Limited, English only
- Offline: ✅, ✅, ❌, ❌, Partial

Link: `Full comparison →` to /compare

### Block 7: Final CTA

**EN:** `Start learning Japanese your way` → /download
**RU:** `Начните изучать японский по-своему` → /download

---

## 5. Features Page (`/features`)

**Structure:** H1 → 4 feature sections (each with H2, How it works steps, Key capabilities bullets) → CTA

**EN H1:** `Everything for Japanese learning`
**RU H1:** `Всё для изучения японского`

### Section 1: Vocabulary

- EN H2: `Vocabulary`
- How it works: Type or paste → Scan a photo → Record audio
- Capabilities: Built-in bilingual dictionaries, instant card creation, NHK audio pronunciation, import from textbooks, FSRS spaced repetition, JLPT level tracking

### Section 2: Kanji

- EN H2: `Kanji`
- How it works: Automatic furigana (fades as you learn) → Writing practice with stroke order → JLPT dictionaries N5–N1 → Interactive reading tests
- Insight: `You don't need to learn all 2,136 jōyō kanji at once. Learn the ones that appear in your content.`

### Section 3: Grammar

- EN H2: `Grammar`
- Capabilities: Structured reference by JLPT level, contextual examples using learned vocabulary, practice tests
- Insight: `Grammar examples use vocabulary you already know — so you focus on the pattern itself.`

### Section 4: Listening & Phrases

- EN H2: `200K+ Native Phrases`
- How it works: N+1 auto-selection, original voice acting, listening comprehension, everyday Japanese
- Insight: `These phrases come from content made by Japanese speakers, for Japanese speakers.`

CTA → /download

---

## 6. Compare Page (`/compare`)

**EN H1:** `Choosing the right Japanese learning tool`
**RU H1:** `Выбор инструмента для изучения японского`

**Subtitle:** `There's no single perfect app. Here's an honest comparison to help you decide.`

### Comparison Table

| Feature | Origa | Anki | WaniKani | Bunpro | Duolingo |
| --- | --- | --- | --- | --- | --- |
| Vocabulary | ✅ | ✅ | ✅ | ❌ | ✅ |
| Kanji | ✅ | Partial | ✅ | ❌ | Basic |
| Grammar | ✅ | ❌ | ❌ | ✅ | ✅ |
| Listening | ✅ | ❌ | ❌ | ❌ | ✅ |
| Your language | ✅ | Manual | English | English | Limited |
| Offline | ✅ | ✅ | ❌ | ❌ | Partial |

Note: "Your language" row header avoids "Languages supported" (which could make Origa look weak vs Anki/Duolingo). Focus is: native language support for non-English learners.

### Per-Competitor Sections

Each section: What is [tool] → When [tool] is right → When Origa is better → Using both together

**Anki:** Powerful general-purpose SRS. Right for: multi-subject, custom HTML/CSS cards. Origa better: all-in-one, native language, zero config. Together: import Anki decks into Origa.

**WaniKani:** Radical-based kanji learning. Right for: structured path from zero. Origa better: learn what you encounter today. Together: WaniKani foundation, then Origa for real-world practice.

**Bunpro:** Grammar drilling. Right for: focused grammar practice. Origa better: vocabulary + grammar unified (examples use words you know). Together: Bunpro for drilling, Origa for unified system.

**Duolingo:** Not a competitor — a starting point. Duolingo for beginners, Origa for growth when Duolingo isn't enough anymore.

CTA → /download

---

## 7. Download Page (`/download`)

**EN H1:** `Get Origa`
**RU H1:** `Получите Origa`

Platform cards with download buttons:

- Windows (.exe, .msi)
- Linux (.deb, .AppImage, .rpm)
- macOS (.dmg, .app)
- Android (.apk)
- Web version (link)

`All platforms support offline mode.`

---

## 8. Privacy & Terms Pages

Legal text pages. Minimal styling. No complex layout.

- `/privacy` — Privacy Policy (EN + RU)
- `/terms` — Terms of Service (EN + RU)
- Content TBD (legal review needed)

---

## 9. Schema.org

| Page | Schema Types |
| --- | --- |
| / | `SoftwareApplication` + `Organization` |
| /features | `SoftwareApplication` + `HowTo` (per feature section) |
| /compare | `SoftwareApplication` + `ItemList` |
| /download | `SoftwareApplication` |

### SoftwareApplication JSON-LD (example)

```json
{
  "@context": "https://schema.org",
  "@type": "SoftwareApplication",
  "name": "Origa",
  "applicationCategory": "EducationalApplication",
  "operatingSystem": "Windows, Linux, macOS, Android, Web",
  "description": "All-in-one Japanese learning app with vocabulary, kanji, grammar and 200K+ native phrases.",
  "featureList": "Vocabulary, Kanji, Grammar, Listening, JLPT Analytics, Offline Mode",
  "inLanguage": ["en", "ru"]
}
```

### HowTo JSON-LD (for /features vocabulary section)

```json
{
  "@context": "https://schema.org",
  "@type": "HowTo",
  "name": "How to build Japanese vocabulary with Origa",
  "step": [
    {"@type": "HowToStep", "text": "Type or paste a Japanese word or sentence"},
    {"@type": "HowToStep", "text": "Scan a photo — OCR extracts text automatically"},
    {"@type": "HowToStep", "text": "Review with spaced repetition at optimal intervals"}
  ]
}
```

---

## 10. Tone of Voice Rules

- Direct address to user ("you" / "вы")
- Explain "why" not just "what"
- Be specific: use numbers ("200,000+ phrases", "N5–N1")
- Respect competitors — never disparage
- No exaggerations or superlatives
- No humor
- **No mentions of: pricing, open source, license, BSL** (`free` / `бесплатно` / `miễn phí` / `무료` is permitted — the app is currently free to use)
- RU: address on "вы", slightly more detailed explanations, keep technical terms (JLPT, FSRS, N+1)

---

## 11. Verification Strategy

- Lighthouse SEO score ≥ 90
- All title tags ≤ 50 chars ✅ (verified)
- All descriptions ≤ 125 chars ✅ (verified)
- hreflang validated via Google International Targeting report
- sitemap.xml submitted to Google Search Console
- All images have alt attributes
- Core Web Vitals: LCP < 2.5s, CLS < 0.1, INP < 200ms
- robots.txt accessible at /robots.txt
- Canonical URLs on every page

---

## 12. Keyword Map

### EN Keywords

| Keyword | Intent | Target Page |
| --- | --- | --- |
| learn Japanese app | Transactional | / |
| Japanese learning app | Transactional | / |
| JLPT preparation app | Transactional | /, /features |
| offline Japanese app | Transactional | / |
| Anki alternative | Comparison | /compare |
| WaniKani alternative | Comparison | /compare |
| learn Japanese in your own language | USP (unique) | / |
| all in one Japanese learning | USP (unique) | / |
| Japanese kanji study | Informational | /features |
| Japanese grammar app | Transactional | /features |

### RU Keywords

| Keyword | Intent | Target Page |
| --- | --- | --- |
| приложение для изучения японского | Transactional | / |
| учить японский язык | Transactional | / |
| подготовка к JLPT | Transactional | /, /features |
| аналог Anki | Comparison | /compare |
| японский язык без английского | USP (unique) | / |
| японские иероглифы учить | Transactional | /features |

---

## 13. Visual Assets — Final Inventory

Landing images live in `origa_landing/public/images/` and are deployed with the site.
The repo-wide media map (app screenshots, store assets, brand art) lives in `.media/README.md`.
Landing images are NOT duplicated in `.media/`.

### Page images

| File | Usage | Page |
| --- | --- | --- |
| `en.hero.png` | Homepage hero image | / (EN) |
| `ru.hero.png` | Homepage hero image | /ru/ |
| `en.all_in_one.png` | Dashboard/overview showcase | /features (EN) |
| `ru.all_in_one.png` | Dashboard/overview showcase | /features (RU) |
| `en.learn.png` | Learning interface showcase | /features, Listening section (EN) |
| `ru.learn.png` | Learning interface showcase | /features, Listening section (RU) |
| `content.png` | Content sources showcase | /integrations (hero image, EN/RU) |

### Meta and feature images

| File | Usage | Page | Size |
| --- | --- | --- | --- |
| `en.og.png` | Social sharing preview (og:image) | All EN pages (meta) | 1729×910 |
| `ru.og.png` | Social sharing preview (og:image) | All RU pages (meta) | 1729×910 |
| `en.kanji.png` | Kanji feature screenshot | /features, Kanji section (EN) | 1448×1086 |
| `ru.kanji.png` | Kanji feature screenshot | /features, Kanji section (RU) | 1448×1086 |
| `en.grammar.png` | Grammar feature screenshot | /features, Grammar section (EN) | 1448×1086 |
| `ru.grammar.png` | Grammar feature screenshot | /features, Grammar section (RU) | 1448×1086 |

### Asset → Page Mapping

```text
/                → en.hero.png / ru.hero.png (hero), en.og.png / ru.og.png (meta)
/features        → en.all_in_one.png (overview), en.kanji.png (kanji section),
                   en.grammar.png (grammar section), en.learn.png (listening section)
/integrations    → content.png (content sources showcase)
/compare         → no images (text table)
/download        → no images (platform icons via CSS/SVG)
/privacy         → no images
/terms           → no images
```

### Static assets in origa_landing/public/ (already in place)

- `favicon.png` → copy of `origa_ui/public/logo-32.png`
- `apple-touch-icon.png`, `favicon.ico`, `browserconfig.xml` → site icons
- `og-image.png` → alias for `en.og.png` (default fallback)
- `robots.txt`, `sitemap.xml` → crawler files
