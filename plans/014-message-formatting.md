# Plan 014: Message Formatting

## Overview

Реализовать форматирование сообщений с тулбаром и поддержкой hotkeys, аналогично Pachka.

## UI Reference (Pachka)

```
┌─────────────────────────────────────────────────────────────────────┐
│  B  I  U  S  │  🔗  │  ≡  ≡  │  "  │  <>  │  [code]  │  Tx        │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  Сообщение...                                                       │
│                                                                     │
├─────────────────────────────────────────────────────────────────────┤
│  📎  😊  @  Aa                                           ▷  │  ∨   │
└─────────────────────────────────────────────────────────────────────┘
```

**Тулбар форматирования (верхний ряд):**
- **B** — Bold (Cmd+B)
- **I** — Italic (Cmd+I)
- **U** — Underline (Cmd+U)
- **S** — Strikethrough (Cmd+Shift+S)
- **🔗** — Link (Cmd+K)
- **≡** — Bulleted list
- **≡** — Numbered list
- **"** — Blockquote
- **<>** — Inline code
- **[code]** — Code block
- **Tx** — Clear formatting

**Нижний ряд:**
- **📎** — Attach files (уже есть)
- **😊** — Emoji picker (future)
- **@** — Mention participant
- **Aa** — Toggle formatting toolbar

---

## Requirements

### Форматирование
| Feature | Hotkey | Markdown | HTML Output |
|---------|--------|----------|-------------|
| Bold | Cmd+B | `**text**` | `<strong>` |
| Italic | Cmd+I | `*text*` | `<em>` |
| Underline | Cmd+U | `__text__` | `<u>` |
| Strikethrough | Cmd+Shift+S | `~~text~~` | `<s>` |
| Link | Cmd+K | `[text](url)` | `<a href>` |
| Bulleted list | — | `- item` | `<ul><li>` |
| Numbered list | — | `1. item` | `<ol><li>` |
| Blockquote | — | `> text` | `<blockquote>` |
| Inline code | Cmd+E | `` `code` `` | `<code>` |
| Code block | Cmd+Shift+C | ```` ``` ```` | `<pre><code>` |
| Clear formatting | Cmd+\ | — | plain text |

### Mentions (@)
- Ввод `@` показывает dropdown со списком участников
- Фильтрация по имени при вводе
- Выбор стрелками + Enter
- Mention сохраняется как `@[user_id]` в контенте
- Отображается как кликабельная ссылка `@Имя`

### Input Area
- Минимальная высота: 1 строка (44px)
- Расширяется автоматически по контенту
- Максимум: 25-30% высоты viewport
- После максимума — внутренний скролл
- Тулбар всегда виден над текстовым полем

---

## Technology Options

### Option A: Tiptap (Recommended)

**Tiptap** — headless WYSIWYG editor на базе ProseMirror.

**Pros:**
- Vue 3 интеграция из коробки
- Модульная архитектура (extensions)
- Полный контроль над UI
- Поддержка Markdown input/output
- Встроенные extensions для всех нужных фич
- Активное сообщество, хорошая документация
- MIT лицензия

**Cons:**
- Относительно большой bundle (~100-150KB gzip)
- Кривая обучения ProseMirror концепций

**Extensions needed:**
- `@tiptap/starter-kit` (bold, italic, lists, code, blockquote)
- `@tiptap/extension-underline`
- `@tiptap/extension-link`
- `@tiptap/extension-mention`
- `@tiptap/extension-placeholder`
- `@tiptap/extension-typography` (smart quotes, etc.)

```bash
npm install @tiptap/vue-3 @tiptap/starter-kit @tiptap/extension-underline @tiptap/extension-link @tiptap/extension-mention @tiptap/extension-placeholder @tiptap/extension-typography
```

**Note:** Markdown shortcuts встроены в starter-kit через `inputRules`. Например, `**bold**` автоматически конвертируется при вводе.

### Option B: Editor.js

**Editor.js** — block-based editor.

**Pros:**
- Чистый JSON output
- Модульная система плагинов
- Хорошо для long-form контента

**Cons:**
- Block-based (не подходит для чата — нужен inline)
- Нет нативной Vue 3 интеграции
- Сложнее адаптировать под chat input
- Меньше контроля над inline форматированием

**Verdict:** Не подходит для chat-style input.

### Option C: Custom (contenteditable + execCommand)

**Pros:**
- Минимальный bundle size
- Полный контроль

**Cons:**
- `execCommand` deprecated
- Много edge cases
- Кросс-браузерные проблемы
- Долгая разработка
- Баги с selection/cursor

**Verdict:** Слишком много работы, много багов.

### Option D: Quill

**Quill** — rich text editor.

**Pros:**
- Популярный, стабильный
- Delta format для контента

**Cons:**
- Monolithic, сложно кастомизировать
- Устаревший дизайн API
- Нет официальной Vue 3 поддержки
- Большой bundle

**Verdict:** Устарел, Tiptap лучше.

---

## Recommended: Tiptap

**Причины выбора:**
1. Vue 3 нативная интеграция
2. Headless — полный контроль над UI (можно сделать как в Pachka)
3. Модульность — только нужные extensions
4. Mention extension из коробки
5. Markdown input parsing
6. Активная разработка

---

## Implementation Plan

### Phase 1: Basic Editor Setup

**Backend:**
- Обновить схему — `messages.content` хранит HTML или Markdown
- Sanitize HTML на бэкенде (ammonia crate)
- API не меняется (content остается string)

**Frontend:**
1. Установить Tiptap и extensions
2. Создать `MessageEditor.vue` компонент
3. Базовое форматирование (bold, italic, underline, strikethrough)
4. Заменить `<input>` на Tiptap editor в MTChat.vue
5. **Markdown shortcuts** — распознавание при вводе:
   - `**text**` → bold
   - `*text*` → italic
   - `~~text~~` → strikethrough
   - `` `code` `` → inline code
   - `> ` в начале строки → blockquote
   - `- ` или `* ` → bullet list
   - `1. ` → numbered list

### Phase 2: Toolbar & Hotkeys

1. Создать `EditorToolbar.vue` с кнопками форматирования
2. Реализовать все hotkeys
3. Toggle состояния кнопок (active state)
4. Кнопка "Aa" для показа/скрытия тулбара
5. Link dialog (Cmd+K)

### Phase 3: Lists, Quotes, Code

1. Bulleted list
2. Numbered list
3. Blockquote
4. Inline code
5. Code block (без подсветки синтаксиса, можно добавить позже)
6. Clear formatting

### Phase 4: Mentions

1. `@tiptap/extension-mention` setup
2. Suggestion dropdown component
3. Фильтрация участников
4. Keyboard navigation
5. Сохранение как `<span data-mention="user_id">@Name</span>`
6. Стилизация mentions в сообщениях

### Phase 5: Auto-resize & Polish

1. Auto-resize textarea (до 25-30%)
2. Внутренний скролл после максимума
3. Mobile-friendly тулбар
4. Dark theme поддержка
5. i18n для тултипов

---

## File Changes

### New Files
```
mtchat-vue/src/components/chat/
├── MessageEditor.vue      # Tiptap editor wrapper
├── EditorToolbar.vue      # Formatting toolbar
├── MentionList.vue        # @ mention dropdown
└── LinkDialog.vue         # Link insert dialog
```

### Modified Files
```
mtchat-vue/src/components/MTChat.vue  # Replace input with MessageEditor
mtchat-vue/src/types/index.ts         # Add Mention type
mtchat-vue/src/i18n/translations.ts   # Toolbar tooltips
mtchat-vue/package.json               # Tiptap dependencies
```

### Backend (optional)
```
mtchat-rust/src/main.rs               # HTML sanitization
mtchat-rust/Cargo.toml                # ammonia crate
```

---

## Data Format

### Storage Format: HTML

Сообщения хранятся как sanitized HTML:

```html
<p>Hello <strong>world</strong>!</p>
<p>Check this <a href="https://example.com">link</a></p>
<ul>
  <li>Item 1</li>
  <li>Item 2</li>
</ul>
<p>Hey <span data-mention="uuid-123" class="mention">@John</span>!</p>
```

### Sanitization (Backend)

Разрешенные теги:
- `p`, `br`
- `strong`, `em`, `u`, `s`
- `a` (href только http/https)
- `ul`, `ol`, `li`
- `blockquote`
- `code`, `pre`
- `span` (только data-mention)

Запрещено:
- `script`, `style`, `iframe`
- `onclick`, `onerror` и прочие event handlers
- `javascript:` URLs

---

## UI Specifications

### Toolbar
```
Height: 40px
Background: var(--mtchat-bg-secondary)
Border-bottom: 1px solid var(--mtchat-border)
Button size: 32x32px
Button spacing: 4px
Separator: 1px vertical line, 8px margin
```

### Editor Area
```
Min height: 44px (1 line)
Max height: 25vh (25% viewport height)
Padding: 12px 16px
Font: 14px, same as messages
Placeholder: "Введите сообщение..." (i18n)
```

### Mention Dropdown
```
Position: above cursor
Max height: 200px
Width: 250px
Item height: 40px
Shows: Avatar + Name + Company
Keyboard: ↑↓ to navigate, Enter to select, Esc to close
```

---

## Hotkeys Summary

| Action | Mac | Windows/Linux |
|--------|-----|---------------|
| Bold | ⌘+B | Ctrl+B |
| Italic | ⌘+I | Ctrl+I |
| Underline | ⌘+U | Ctrl+U |
| Strikethrough | ⌘+⇧+S | Ctrl+Shift+S |
| Link | ⌘+K | Ctrl+K |
| Inline code | ⌘+E | Ctrl+E |
| Code block | ⌘+⇧+C | Ctrl+Shift+C |
| Clear format | ⌘+\ | Ctrl+\ |
| Send message | ⌘+Enter or Enter | Ctrl+Enter or Enter |

---

## Migration

### Existing Messages
- Старые сообщения (plain text) отображаются как есть
- При редактировании конвертируются в HTML: `<p>{content}</p>`
- Не требуется миграция БД

### Backward Compatibility
- API не меняется (`content: string`)
- Frontend определяет формат по наличию HTML тегов
- Plain text рендерится через `white-space: pre-wrap`

---

## Estimated Effort

| Phase | Effort |
|-------|--------|
| Phase 1: Basic Editor | 4-6h |
| Phase 2: Toolbar & Hotkeys | 4-6h |
| Phase 3: Lists, Quotes, Code | 3-4h |
| Phase 4: Mentions | 4-6h |
| Phase 5: Polish | 2-3h |
| **Total** | **17-25h** |

---

## Decisions

1. **Emoji picker** — отдельная задача, позже
2. **Syntax highlighting** — не обязательно, можно добавить позже
3. **Markdown input** — ✅ да, распознавать при вводе/вставке (Tiptap имеет `@tiptap/extension-typography` и можно добавить markdown shortcuts)
4. **File drag & drop** — отдельная задача, позже

---

## References

- [Tiptap Documentation](https://tiptap.dev/)
- [Tiptap Vue 3 Guide](https://tiptap.dev/installation/vue3)
- [Tiptap Mention Extension](https://tiptap.dev/api/nodes/mention)
- [ammonia (Rust HTML sanitizer)](https://docs.rs/ammonia/)

---

*Plan created 2025-02-06*
