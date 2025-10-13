# Стиль приложения (React95)

Это руководство описывает правила оформления интерфейсов в проекте на основе библиотеки React95. Документ написан для разработчиков и агентов-ассистентов: здесь перечислены инварианты, договорённости и готовые рецепты. Официальная документация: [React95 Components & Themes](https://react95.github.io/React95/).

---

## 🎯 Цели стиля

- Воссоздать эстетику Windows 95 с помощью компонентов React95
- Использовать централизованные CSS‑переменные вместо хардкодов
- Поддерживать единый точечный вход для темизации и глобальных стилей
- Минимизировать «ручные» стили, предпочитая готовые компоненты

---

## 🔌 Базовая интеграция

Глобальные стили и тема подключаются один раз в точке входа приложения:

```startLine:endLine:jeels/src/main.tsx
import '@react95/core/GlobalStyle';
import '@react95/core/themes/win95.css';
```

Этого достаточно, чтобы все компоненты React95 выглядели корректно. При необходимости можно заменить тему (`win95.css`) на другую из каталога тем, см. витрину тем в оф. документации: [Themes](https://react95.github.io/React95/).

---

## 🎨 Переменные темы проекта

Все пользовательские цвета определяются в одном месте — в `App.css` в корне (`:root`). Эти переменные могут переопределяться на уровне страницы/виджета при необходимости.

```startLine:endLine:jeels/src/App.css
:root {
  --background: rgb(85, 170, 170);
  --foreground: #171717;
  --window-bg: #c0c0c0;
  --window-border: #c0c0c0;
  --message-bg: #f0f0f0;
  --message-border: #999;
  --link-color: #0000ff;
  --link-hover: #ff0000;
  --link-visited: #800080;
}

body,
#root {
  background: var(--background) !important;
  font-family: 'MS Sans Serif', sans-serif;
}
```

Правила:

- Никаких «жёстких» значений цветов в JSX; всегда используем переменные.
- Если глобальные стили React95 переопределяют фон/цвет, добавляем более специфичный селектор или `!important` на уровне проекта.

---

## 🧱 Рекомендуемые компоненты (из React95)

- Контейнеры и секции: `Fieldset`, `Frame`
- Формы: `Input`, `TextArea`, `Checkbox`, `RadioButton`, `Range`, `Dropdown`, `Button`
- Навигация и вспомогательные элементы: `List`, `Tabs`/`Tab`, `Tooltip`, `TaskBar`, `TitleBar`
- Медиа и прогресс: `Video`, `ProgressBar`
- Окна/рамки/диалоги: `Modal`, `Alert`
- Иконки и графика: `Icon`, `Avatar`, `Cursor`
- Хуки: `useModal`

См. каталог компонентов: [Components](https://react95.github.io/React95/).

---

## 📚 Полный список компонентов (с импорта)

Ниже перечислены компоненты, доступные в проекте, с типичным импортом. Актуальный перечень и API уточняйте по витрине: [react95.github.io/React95](https://react95.github.io/React95/).

```tsx
import {
  Alert,
  Avatar,
  Button,
  Checkbox,
  Contract,
  Cursor,
  Dropdown,
  Fieldset,
  Frame,
  GlobalStyle,
  Icon,
  Input,
  List,
  Modal,
  ProgressBar,
  RadioButton,
  Range,
  Tabs, Tab,
  TaskBar,
  TextArea,
  TitleBar,
  Tooltip,
  Tree,
  Video,
} from '@react95/core';

import { useModal } from '@react95/core';
```

Краткие назначения:

- **Alert**: простые уведомления/диалоги подтверждения
- **Avatar**: аватары пользователей/объектов
- **Button**: кнопки действий
- **Checkbox**: чекбоксы
- **Contract**: служебный декоративный блок/контейнер (см. витрину)
- **Cursor**: стилизованный курсор/указатель
- **Dropdown**: выпадающие списки/меню
- **Fieldset**: логические группы UI c легендой
- **Frame**: рамки/панели с 95‑стилем
- **GlobalStyle**: глобальные базовые стили React95
- **Icon**: иконки интерфейса
- **Input**: однострочный ввод
- **List**: списки элементов (навигация/контент)
- **Modal**: модальные окна
- **ProgressBar**: индикатор выполнения
- **RadioButton**: радио‑переключатели
- **Range**: ползунок диапазона
- **Tabs/Tab**: вкладки
- **TaskBar**: нижняя панель в стиле Windows 95
- **TextArea**: многострочный ввод
- **TitleBar**: заголовок окна/панели
- **Tooltip**: подсказки
- **Tree**: иерархические списки
- **Video**: встроенное видео/контейнер
- **useModal**: хук управления модалками (открыть/закрыть)

Пример связки `useModal` + `Modal`:

```tsx
// псевдокод, уточняйте API по витрине компонентов
const { isOpen, open, close } = useModal();

return (
  <>
    <Button onClick={open}>Open modal</Button>
    <Modal open={isOpen} onClose={close}>
      <Fieldset legend="Dialog">...</Fieldset>
    </Modal>
  </>
);
```

---

## 📦 Образец страницы

Фрагмент, демонстрирующий рекомендованные приёмы: переменные, компоненты React95 и отсутствие хардкодов.

```startLine:endLine:jeels/src/App.tsx
<div style={{
  padding: '20px',
  background: 'var(--background)',
  minHeight: '100vh',
  display: 'flex',
  flexDirection: 'column',
  gap: '20px',
  alignItems: 'center'
}}>
  <div style={{
    width: '100%',
    maxWidth: '600px',
    background: 'var(--window-bg)',
    border: '2px outset var(--window-border)',
    padding: '20px'
  }}>
    <Fieldset legend="About">...</Fieldset>
  </div>
</div>
```

---

## 🖌️ Правила оформления

- Отступы и сетка — через flex и стандартные отступы, без магических чисел
- Текст и ссылки наследуют цвета из переменных (`--foreground`, `--link-*`)
- Любые «карточки», панели и сообщения используют `--window-bg`, `--window-border`, `--message-*`
- Иконки берём из `@react95/icons` и не задаём им размеры в пикселях без необходимости; используем вспомогательные классы `.w-4`, `.h-4`

---

## 🌗 Темизация и расширение тем

- Базовую тему даёт CSS-файл из React95 (`win95.css`).
- Пользовательская палитра управляется нашими переменными в `:root`.
- Для альтернативных схем создайте дополнительные корневые селекторы (например, `[data-theme="alt"]`) и переопределите переменные.

Пример переключения тем на уровне корня приложения:

```css
/* baseline */
:root { --background: rgb(85, 170, 170); /* ... */ }

/* альтернативная тема */
[data-theme='alt'] {
  --background: #9cc3ff;
  --foreground: #0f172a;
}
```

---

## 🔗 Ссылки

- Витрина и документация компонентов/тем: [react95.github.io/React95](https://react95.github.io/React95/)
- Пакеты: `@react95/core`, `@react95/icons`

---

## ✅ Чек‑лист для PR

- Нет хардкод‑цветов в JSX/TSX — используются только CSS‑переменные
- Компоненты — из React95, без самодельных аналогов там, где есть готовые
- Глобальные стили подключены в точке входа
- Новые цвета/акценты добавлены в `:root` и задокументированы
