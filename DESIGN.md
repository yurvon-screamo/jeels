---
version: alpha
name: Origa Design System
colors:
  primary: "#3d4535"
  secondary: "#a65d3f"
  tertiary: "#8b7355"
  neutral: "#6b665a"
  surface: "#f7f4ee"
  on-surface: "#1a1915"
  error: "#8b4545"
  success: "#4a6b4a"
  warning: "#a67c3f"
  bg-cream: "#f7f4ee"
  bg-paper: "#fdfbf7"
  bg-aged: "#ede8dd"
  bg-warm: "#f0ebe1"
  fg-black: "#1a1915"
  fg-muted: "#6b665a"
  fg-light: "#9a9590"
  border-light: "#d4cfc3"
  border-dark: "#1a1915"
  accent-olive: "#3d4535"
  accent-terracotta: "#a65d3f"
  accent-gold: "#8b7355"
  accent-sage: "#6b7b5e"
typography:
  headline-lg:
    fontFamily: "Cormorant Garamond"
    fontSize: "48px"
    fontWeight: 300
    lineHeight: 1.1
    letterSpacing: "-0.01em"
  headline-md:
    fontFamily: "Cormorant Garamond"
    fontSize: "32px"
    fontWeight: 400
    lineHeight: 1.2
  headline-sm:
    fontFamily: "Cormorant Garamond"
    fontSize: "24px"
    fontWeight: 500
    lineHeight: 1.2
  body-lg:
    fontFamily: "IBM Plex Mono"
    fontSize: "16px"
    fontWeight: 400
    lineHeight: 1.6
    letterSpacing: "0.02em"
  body-md:
    fontFamily: "IBM Plex Mono"
    fontSize: "14px"
    fontWeight: 400
    lineHeight: 1.6
    letterSpacing: "0.05em"
  label-md:
    fontFamily: "IBM Plex Mono"
    fontSize: "13px"
    fontWeight: 400
    lineHeight: 1
    letterSpacing: "0.1em"
    textTransform: uppercase
  label-sm:
    fontFamily: "IBM Plex Mono"
    fontSize: "11px"
    fontWeight: 400
    lineHeight: 1
    letterSpacing: "0.15em"
    textTransform: uppercase
rounded:
  none: 0px
  sm: 2px
  md: 4px
  lg: 8px
spacing:
  xs: 4px
  sm: 8px
  md: 16px
  lg: 24px
  xl: 32px
  xxl: 48px
  xxxl: 64px
border:
  width-default: 1px
  width-thick: 2px
  color-default: "{colors.border-dark}"
  color-light: "{colors.border-light}"
shadow:
  offset-x: 6px
  offset-y: 6px
  color: "{colors.border-dark}"
components:
  button-primary:
    backgroundColor: "{colors.fg-black}"
    textColor: "{colors.bg-paper}"
    borderWidth: "1px"
    borderColor: "{colors.fg-black}"
    borderRadius: "{rounded.none}"
    padding: "10px 16px"
    fontFamily: "{typography.label-md.fontFamily}"
    fontSize: "11px"
    textTransform: uppercase
    letterSpacing: "0.1em"
  button-secondary:
    backgroundColor: "{colors.bg-paper}"
    textColor: "{colors.fg-black}"
    borderWidth: "1px"
    borderColor: "{colors.border-dark}"
    borderRadius: "{rounded.none}"
    padding: "10px 16px"
    fontFamily: "{typography.label-md.fontFamily}"
    fontSize: "11px"
  button-olive:
    backgroundColor: "{colors.accent-olive}"
    textColor: "{colors.bg-paper}"
    borderWidth: "1px"
    borderColor: "{colors.accent-olive}"
  input-field:
    backgroundColor: "{colors.bg-paper}"
    textColor: "{colors.fg-black}"
    borderWidth: "1px"
    borderColor: "{colors.border-dark}"
    borderRadius: "{rounded.none}"
    padding: "14px 18px"
    fontFamily: "{typography.body-md.fontFamily}"
    fontSize: "14px"
    focusShadow: "4px 4px 0 {colors.border-dark}"
  card:
    backgroundColor: "{colors.bg-paper}"
    borderWidth: "1px"
    borderColor: "{colors.border-dark}"
    padding: "28px"
    shadowOffsetX: "6px"
    shadowOffsetY: "6px"
    shadowBorderWidth: "1px"
    shadowBackground: "{colors.bg-aged}"
  tag:
    backgroundColor: "{colors.bg-paper}"
    textColor: "{colors.fg-black}"
    borderWidth: "1px"
    borderColor: "{colors.border-dark}"
    padding: "8px 14px"
    fontFamily: "{typography.label-md.fontFamily}"
    fontSize: "11px"
    textTransform: uppercase
    letterSpacing: "0.2em"
  tag-olive:
    backgroundColor: "{colors.accent-olive}"
    textColor: "{colors.bg-paper}"
    borderColor: "{colors.accent-olive}"
  tag-terracotta:
    backgroundColor: "{colors.accent-terracotta}"
    textColor: "{colors.bg-paper}"
    borderColor: "{colors.accent-terracotta}"
  modal-backdrop:
    backgroundColor: "rgba(26, 25, 21, 0.6)"
  modal-content:
    backgroundColor: "{colors.bg-paper}"
    borderWidth: "1px"
    borderColor: "{colors.border-dark}"
    borderRadius: "{rounded.none}"
    padding: "36px"
    maxWidth: "640px"
    shadowOffset: "8px"
    shadowBackground: "{colors.bg-aged}"
animation:
  duration-fast: "100ms"
  duration-normal: "200ms"
  duration-slow: "300ms"
  duration-entrance: "350ms"
  ease-paper: "cubic-bezier(0.25, 0.46, 0.45, 0.94)"
  ease-out-expo: "cubic-bezier(0.16, 1, 0.3, 1)"
  ease-out-back: "cubic-bezier(0.34, 1.56, 0.64, 1)"
---

# DESIGN.md — Origa Design System

## Overview

Origa — минималистичный интерфейс для приложения изучения
японского языка, вдохновлённый эстетикой японской типографики
и традиционной бумаги. Дизайн отличается низким визуальным
шумом, чёткой геометрией, тёплой землистой палитрой и
авторитетным моноширинным типографическим голосом.

**Концепция**: "Типографика печатного станка на японской бумаге".
Все интерактивные элементы используют 1px чёрные границы без
скруглений. Тени реализованы через псевдо-элементы `::after`
со смещением и линией границы — эффект "сдвига" вместо
размытия. Типографика — это контраст между элегантным serif
(Cormorant Garamond) для заголовков/японского текста и
утилитарным monospace (IBM Plex Mono) для всего интерфейса.

## Colors

Палитра построена на тёплых кремовых фонах
(`bg-cream`, `bg-paper`, `bg-aged`) с почти чёрным текстом
(`fg-black`). Акцентные цвета — приглушённые оливковый,
терракотовый и золотой, которые создают ощущение
традиционных японских красок без яркости.

- **Primary** (`#3d4535`): CTA-кнопки, призыв к действию,
  статус "в процессе", активные состояния. Используется
  умеренно — только когда нужен цветовой акцент.
- **Secondary** (`#a65d3f`): Предупреждения,
  "штамп"-эффекты, hover-ссылки. Визуально заметен,
  привлекает внимание к важным меткам.
- **Tertiary** (`#8b7355`): Менее важные акценты, вторичные метаданные.
- **Surface** (`#f7f4ee`): Основной фон приложения.
  Тёплый, не чисто белый.
- **On-surface** (`#1a1915`): Основной читаемый текст.
  Почти чёрный, но с тёплым подтоном.

### Цветовые правила

1. **Границы всегда чёрные**: Даже при использовании
   цветных вариантов кнопок (olive, terracotta),
   hover-эффекты возвращаются к чёрно-белому контрасту.
2. **Не используйте `border-radius`**: Компоненты имеют
   углы 0px. Единственное исключение — radio (50%) и
   небольшой `border-radius: 2px` для utility-элементов.
3. **Не используйте `box-shadow` с размытием**: Все тени —
   "hard shadows" через offset pseudo-elements.

## Typography

### CSS Variables (Typography Scale)

Все размеры шрифтов используют CSS custom properties из `:root`:

- `--text-2xs: 11px` — мельчайшие лейблы
- `--text-label-sm: 11px` — малые лейблы, кнопки mobile
- `--text-label: 13px` — основные лейблы, sidebar, dropdown
- `--text-body: 14px` — основной текст, input, table
- `--text-sm: 16px` — увеличенный текст
- `--text-xs: 14px` — utility class

### Система шрифтов

Все 4 семейства self-hosted на CDN (`<cdn>/fonts/`) через `@font-face`
с content-hash-версионированием имён (ADR-028). Это даёт офлайн-работу
после первой загрузки и гарантирует японские, а не китайские, глифы CJK.

CSS-переменные задают fallback-цепочку по принципу «Latin-шрифт → CJK-шрифт»:

```css
--font-serif: "Cormorant Garamond", "Noto Sans JP", sans-serif;
--font-mono: "IBM Plex Mono", "Noto Sans JP", monospace;
```

Браузер выбирает шрифт посимвольно: Latin/Cyrillic рендерится
Cormorant/IBM Plex Mono (включают Latin + Cyrillic глифы), а кандзи/кана
«проваливается» к **Noto Sans JP** через `unicode-range` в `@font-face`. Это критично:
Xiaomi MIUI/HyperOS и Android по умолчанию отдают **китайский** CJK-serif
(например MiSans CJK SC) для общих хань-кандзи — формы штрихов 食/海/語/難
визуально неправильны для японского. Noto Sans JP даёт корректные японские
формы. Noto Sans JP также использует `font-display: block`,
чтобы до загрузки шрифта браузер не «мигал» китайским системным CJK.

> **Unified CJK (Gothic):** оба стека (`--font-serif` и `--font-mono`)
> используют единую CJK-семью — **Noto Sans JP**. Ранее `--font-serif`
> падал на Noto Serif JP (Mincho), а `--font-mono` — на Noto Sans JP (Gothic);
> это порождало смешение Mincho-кандзи и Gothic-кана в одном слове
> (display/heading serif + body mono сходились на карточках), что особенно
> ломало верстку на Xiaomi. Latin-сплит сохранён: Cormorant для serif-элементов,
> IBM Plex Mono для mono-элементов. Supersedes ADR-028 §font-set.

| Семейство | Назначение | Вара | Покрытие |
| --------- | ---------- | ---- | -------- |
| Cormorant Garamond | Display/Heading | 300–700 (variable) + italic | Latin, Latin-Extended, **Cyrillic**, пунктуация |
| IBM Plex Mono | Body/Label/Mono | 300, 400, 500 + italic 400 | Latin, Latin-Extended (Vietnamese), **Cyrillic**, пунктуация |
| Noto Sans JP | CJK-fallback для обоих стеков (`--font-serif`, `--font-mono`) | 400 (Regular) | кандзи + кана + пунктуация (subset по корпусу приложения) |

> **Cyrillic support** (ADR-030): Cormorant Garamond и IBM Plex Mono
> теперь subsetted с Cyrillic диапазоном. Ранее Cyrillic UI-текст рендерился
> системным fallback-шрифтом (на Xiaomi — sans-serif), что ломало визуальную
> консистентность интерфейса. DM Mono был заменён на IBM Plex Mono именно
> потому, что DM Mono не имеет Cyrillic глифов.

- **Display**: Cormorant Garamond, 300,
  clamp(1.75rem, 6vw, 3rem) — H1, hero titles
- **Heading**: Cormorant Garamond, 400-500,
  1.25–1.5rem — H2–H6 (Japanese glyphs → Noto Sans JP, Gothic)
- **Body**: IBM Plex Mono, 400, 14px —
   Основной текст, описания
- **Label**: IBM Plex Mono, 400, 11–13px —
   Кнопки, теги, навигация
- **Mono**: IBM Plex Mono, 400-500, 13px —
   Код, технические данные

> Noto Sans JP грузится только Regular (400). С унификацией CJK на Gothic
> (ранее Mincho для serif-стека) риск слишком светлых CJK-глифов рядом с
> Cormorant-500 ВЫШЕ — Gothic 400 оптически легче Mincho 400. Если визуальный
> smoke выявит слишком светлые CJK-заголовки, добавляется Medium (500) —
> задокументировано как follow-up в ADR-028 (теперь приоритетнее).

### Правила типографики

- **Заголовки** (Cormorant Garamond) используют
  `letter-spacing: -0.01em` для естественной плотности.
- **Интерфейсные элементы** (IBM Plex Mono) используют
  `text-transform: uppercase` + `letter-spacing: 0.1em` —
  это создаёт эффект печатного набора.
- **Размеры шрифта для UI-компонентов** никогда не превышают
  14px — приложение использует очень мелкий, плотный
  типографический масштаб.

### Офлайн и кэш шрифтов

Шрифты деплоятся на CDN с `Cache-Control: public, max-age=31536000, immutable`
(truly-static). После первого онлайн-запуска WebView кэширует woff2, и
последующие запуски (в т.ч. офлайн) рендерят CJK корректно. Полный
офлайн на самом первом запуске требует bundle в `tauri/resources/fonts/`
(follow-up, ADR-028). Регенерация subset — `uv run --with "fonttools[woff]"
scripts/subset_fonts.py` (подробности в `cdn/README.md`).

## Layout & Spacing

### Принципы компоновки

- **No border-radius**: Все контейнеры — строго прямоугольные.
- **Hard shadows**: Карточки и модальные окна имеют "тень"
  как отдельный offset-слой с границей, а не CSS box-shadow.
- **Отступы**: Используется 8px сетка (4, 8, 16, 24, 32, 48, 64, 96px).
- **Контейнеры**:
  - `card-layout-small`: max-width 480px
  - `card-layout-medium`: max-width 768px
  - `card-layout-large`: max-width 1024px
  - `card-layout-lesson`: max-width 1280px — интерактивные карточки вопроса/ответа в `/lesson`. Шире `card-layout-large`, потому что карточка заполняет viewport по вертикали (ADR-031 stable min-h через svh) и при прежнем `max-w-4xl` (896px) на десктопе оставляла слишком много пустого места по краям. Quiz options внутри используют 3-колоночный grid на ≥768px (`md:grid-cols-3`).
  - `card-layout-adaptive`: width 100%

### Responsive Breakpoints

| Breakpoint | Ширина     | Ключевые изменения                          |
| ---------- | ---------- | ------------------------------------------- |
| Mobile     | < 640px    | Single column, bottom nav, compact padding  |
| Tablet     | 640–1023px | Multi-column grids, medium padding (24px)   |
| Desktop    | ≥ 1024px   | Full layouts, large padding (32px)          |

**Safe Areas**: Все fixed-элементы учитывают
`env(safe-area-inset-*)` для iPhone notch.

## Elevation & Depth

Все эффекты глубины достигаются через
**псевдо-элементы** с чёткими границами, а не blur-модели.

```text
Card Shadow:
┌──────────────┐
│   Content    │
│              │
└──────────────┘
         ┌──────────────┐
         │  1px border  │  ← Offset 6px right, 6px bottom
         │  bg-aged     │
         └──────────────┘
```

- **Card shadow**:
  `bottom: -6px; right: -6px; border: 1px solid
  var(--border-dark); background: var(--bg-aged)`
- **Modal shadow**: `bottom: -8px; right: -8px;`
- **Toast shadow**:
  `box-shadow: 4px 4px 0 var(--border-dark)`
  (единственное место с box-shadow — toast)

## Shapes

- **Buttons**: Прямоугольники,
   padding `10px 16px` (mobile) / `16px 32px` (desktop).
- **Inputs**: Прямоугольники, focus-эффект —
   `box-shadow: 4px 4px 0 var(--border-dark)`.
- **Tags**: Inline-flex, прямоугольные,
   плотный letter-spacing.
- **Checkbox/Radio**: Square (22×22) / Circle (22×22),
   checked через inset square/circle.
- **Toggle**: Square track (48×28),
   square thumb with border.

**Запрещено**: `border-radius` на основных компонентах
(карточки, кнопки, модальные окна, формы).

## Components

### Button

```css
.btn {
  font-family: "IBM Plex Mono", monospace;
  font-size: 11px;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  padding: 10px 16px;
  border: 1px solid var(--border-dark);
  background: var(--bg-paper);
  color: var(--fg-black);
  cursor: pointer;
  transition: all 0.2s ease;
  display: inline-flex;
  align-items: center;
  justify-content: center;
}
```

Варианты:

- **Default**: Белый фон, чёрная граница.
  Hover: инверсия (чёрный фон, белый текст).
- **Filled**: Чёрный фон, белый текст. Hover: инверсия.
- **Olive**: Оливковый фон/граница. Hover: белый фон, оливковый текст.
- **Ghost**: Прозрачный фон, без границы на покое. Hover: белый фон + граница.

Hover-эффект: псевдо-элемент `::before` с
`inset: 2px` и `border: 1px` появляется с opacity 0→1.

### Card

```css
.card {
  background: var(--bg-paper);
  border: 1px solid var(--border-dark);
  padding: 28px;
  position: relative;
}
```

- `.card-shadow` — добавляет offset shadow через `::after`.
- `.card-selectable` — hover меняет фон на `bg-aged`.
- `.selected` — граница `accent-olive`, фон `bg-warm`.

### Modal

```css
.modal-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(26, 25, 21, 0.6);
  z-index: 100;
}

.modal-content {
  position: fixed;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  background: var(--bg-paper);
  border: 1px solid var(--border-dark);
  padding: 36px;
  max-width: 640px;
  width: 90%;
  z-index: 101;
}
```

Модальное окно имеет offset shadow (`::after`) и
поддерживает анимации входа/выхода
(`anima-modal-enter` / `anima-modal-exit`).
Закрывается по Escape и click на backdrop.

### Drawer

- **Mobile** (<640px): выезжает снизу, height 80vh, border-top.
- **Desktop** (≥640px): выезжает справа, width 80vw max 800px, border-right.
- Оба варианта имеют offset shadow и анимацию slide.

### Input

```css
.input-field {
  font-family: "IBM Plex Mono", monospace;
  font-size: 14px;
  padding: 14px 18px;
  border: 1px solid var(--border-dark);
  background: var(--bg-paper);
  color: var(--fg-black);
  width: 100%;
}

.input-field:focus {
  outline: none;
  box-shadow: 4px 4px 0 var(--border-dark);
}

.input-field::placeholder {
  color: var(--fg-muted);
  text-transform: uppercase;
  font-size: 12px;
  letter-spacing: 0.1em;
}
```

Placeholder всегда uppercase с letter-spacing — это фирменный паттерн.

### Tag

Категориальная метка. Бывает интерактивной (фильтр/переключатель)
и неинтерактивной (статус/категория).

**Non-interactive Tag** (без `on_click`):

```css
.tag {
  font-family: "IBM Plex Mono", monospace;
  font-size: 11px;
  letter-spacing: 0.2em;
  text-transform: uppercase;
  padding: 8px 14px;
  border: 1px solid var(--border-light);
  background: var(--bg-paper);
  color: var(--fg-muted);
}
```

Светлая рамка и тусклый текст — визуально "лёгкий" элемент.
Не имеет hover-эффектов. Рендерится как `<span>`.

**Clickable Tag** (с `on_click`, класс `tag-clickable`):

```css
.tag-clickable {
  border-color: var(--border-dark);
  color: var(--fg-black);
  cursor: pointer;
}
.tag-clickable:hover {
  background: var(--fg-black);
  color: var(--bg-paper);
  transform: translateY(-1px);
}
.tag-clickable:active {
  transform: translateY(1px) scale(0.98);
}
```

Чёрная рамка, color flip hover, press-эффект на active.
Рендерится как `<button>`. Имеет `::before` inner border на hover
и `anima-focus-ring` для keyboard navigation.

Варианты: `tag-filled`, `tag-olive`, `tag-terracotta`, `tag-sage`.
Все варианты инвертируются на hover (color flip).

### Badge

Неинтерактивный статус-индикатор. Никогда не кликабельный.

```css
.badge {
  font-family: "IBM Plex Mono", monospace;
  font-size: 14px;
  letter-spacing: 0.05em;
  padding: 4px 10px;
  border: none;
  background: var(--bg-aged);
  color: var(--fg-muted);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 20px;
}
```

Badge — это "штамп на полях". Без рамки, приглушённые цвета.
Никогда не имеет `cursor: pointer` или hover-эффектов.

### Interaction Hierarchy

Визуальная шкала интерактивности (Paper Gradient).
Чем "тяжелее" элемент визуально, тем он интерактивнее.

<!-- markdownlint-disable MD013 -->
| Уровень | Компонент            | Рамка          | Фон       | Текст      | Интерактивный |
|---------|----------------------|----------------|-----------|------------|---------------|
| 0       | Inline               | None           | None      | `fg-muted` | Нет           |
| 1       | Badge                | None           | `bg-aged` | `fg-muted` | Нет           |
| 2       | Tag                  | `border-light` | `bg-paper`| `fg-muted` | Нет           |
| 3       | Chip (tag-clickable) | `border-dark`  | `bg-paper`| `fg-black` | Да            |
| 4       | Button               | `border-dark`  | `bg-paper`| `fg-black` | Да            |
<!-- markdownlint-enable MD013 -->

Принцип: чёрная рамка (`border-dark`) = можно нажать.
Светлая рамка или отсутствие рамки = нельзя нажать.

### Furigana

```css
.furigana-ruby {
  /* font-family inherits the container (serif or mono stack) */
  ruby-align: center;
}

.furigana-rt {
  /* font-family inherits the container */
  font-size: 0.6em;
  color: var(--fg-muted);
}

.furigana-plain {
  display: inline;
}
```

Фуригана наследует font-family контейнера: в markdown-теле (`--font-mono`)
кандзи/кана/аннотация — все Noto Sans JP; в заголовках (`--font-serif`) — тоже
единая семья. Per-glyph резолв через `unicode-range` сохранён. Ранее
`.furigana-ruby` жёстко задавал serif, а `.furigana-rt` — mono, что порождало
смешение Mincho-кандзи и Gothic-кана/аннотации в одном слове. Furigana (кана) —
0.6em от размера базового текста.

## Animation System

### Naming Convention

Все анимации используют префикс `anima-*`. Это позволяет
быстро находить анимации в коде и применять их через CSS-классы.

### Категории

- **Standard**: `anima-press` (кнопки), `anima-lift` (карточки),
  `anima-reveal` (навигация) — унифицированные hover/active эффекты
- **Hover**: `anima-card-lift` (deprecated→anima-lift),
  `anima-tag-hover` (deprecated→tag-clickable),
  `anima-link-typewriter`, `anima-btn-press` (deprecated→anima-press),
  `anima-avatar-hover` — Микро-интеракции при наведении
- **Entrance**: `anima-modal-enter`,
  `anima-backdrop-enter`, `anima-toast-bounce`,
  `anima-slide-up` — Появление элементов
- **Exit**: `anima-modal-exit`, `anima-backdrop-exit`,
  `anima-toast-exit` — Исчезновение элементов
- **Micro**: `anima-shake`, `anima-check-draw`,
  `anima-strikethrough` — Отклик на действия
- **Loading**: `anima-skeleton-paper`,
  `anima-spinner-square` — Состояния загрузки
- **Japanese**: `anima-kanji-reveal`,
  `anima-furigana-show` — Для японского языка
- **Layout**: `anima-stagger`, `anima-page-fade`,
  `anima-slide-up` — Каскадное появление
- **Special**: `anima-stamp`, `anima-pulse-subtle`,
  `anima-focus-ring` — Эффекты внимания и фокуса

### Easing Functions

- `anima-ease-paper`:
  `cubic-bezier(0.25, 0.46, 0.45, 0.94)` —
  Стандартные transitions
- `anima-ease-out-expo`:
  `cubic-bezier(0.16, 1, 0.3, 1)` —
  Entrance анимации
- `anima-ease-out-back`:
  `cubic-bezier(0.34, 1.56, 0.64, 1)` —
  Bounce эффекты (toast)

### Standard Animation Classes

Три уровня анимаций для разных семантических ролей:

- **anima-press** — кнопки и действия.
  Физическая метафора: клавиша пишущей машинки.
  Active: `translateY(1px) scale(0.98)`.
  Применяется к: Button, IconBtn, Pagination.

- **anima-lift** — карточки и объекты.
  Физическая метафора: лист бумаги приподнимается.
  Hover: `translateY(-2px)`. Active: `translateY(0)`.
  Применяется к: Card (interactive).

- **anima-reveal** — навигация и ссылки.
  Физическая метафора: маркер выделяет пункт.
  Только opacity/color/bg transition, без transform.
  Применяется к: Sidebar, BottomTab, Breadcrumbs.

### Accessibility

```css
@media (prefers-reduced-motion: reduce) {
  *, *::before, *::after {
    animation-duration: 0.01ms !important;
    transition-duration: 0.01ms !important;
  }
}
```

## Do's and Don'ts

### ✅ Do

- Используйте `1px solid var(--border-dark)` для границ всех компонентов.
- Применяйте `text-transform: uppercase` + `letter-spacing`
  к всему интерфейсному тексту (кнопки, теги, навигация).
- Используйте `Cormorant Garamond` только для заголовков и японского текста.
- Добавляйте offset shadow через `::after` для карточек и модальных окон.
- Используйте `bg-paper` для контента поверх `bg-cream`.
- Применяйте `anima-focus-ring` для доступной навигации с клавиатуры.

### ❌ Don't

- **Не используйте `border-radius`** на основных компонентах
  (кнопки, карточки, модалки, формы). Единственные
  исключения: checkbox/radio активные индикаторы и
  utility-классы.
- **Не используйте `box-shadow` с blur** (например,
  `box-shadow: 0 4px 6px rgba(...)`). Все тени — hard offset.
- **Не используйте яркие цвета** (pure red/green/blue).
  Акцентные цвета Origa — всегда muted/приглушённые.
- **Не используйте шрифты sans-serif** (Inter, Roboto) —
  только Cormorant Garamond и IBM Plex Mono.
- **Не создавайте полупрозрачные фоны** для overlay, кроме
  `modal-backdrop` и `loading-overlay`.
- **Не задавайте font-size > 14px** для интерфейсных элементов
  (кнопки, навигация, теги). Body-текст: 14px, labels: 11–13px.

## Paper Texture

Фоновая текстура "бумаги" реализована через fixed
pseudo-element с SVG noise:

```css
.paper-texture::before {
  content: "";
  position: fixed;
  inset: 0;
  background-image: url("data:image/svg+xml,...feTurbulence...");
  opacity: 0.03;
  pointer-events: none;
  z-index: 0;
}
```

Это создаёт едва заметную зернистость, придающую теплоту
интерфейсу. Элемент применяется к `<body>`.

## Special Patterns

### Stamp Effect

```css
.stamp {
  font-family: "Cormorant Garamond", serif;
  font-size: 13px;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  padding: 8px 16px;
  border: 2px solid var(--accent-terracotta);
  color: var(--accent-terracotta);
  transform: rotate(-3deg);
}
```

Используется для важных меток/статусов. Имеет анимацию
"штамп" (`anima-stamp`) с bounce.

### Kanji Drawing Animation

```css
.kanji-animation-svg path:not(.bg) {
  stroke-dasharray: 1000;
  stroke-dashoffset: 1000;
  animation: kanji-draw 0.4s linear forwards;
}
```

SVG анимация порядка начертания иероглифа — последовательное "рисование" черт.

### Quiz Options

```css
.quiz-option-neutral   /* default */
.quiz-option-correct   /* success color */
.quiz-option-wrong     /* error color */
.quiz-option-dimmed    /* opacity 0.5 */
```

Три состояния для викторин: нейтральное, правильный ответ
(зелёная граница + bg-warm), неправильный ответ
(красная граница + bg-warm). Неправильный ответ
подсвечивается, остальные диммируются.
