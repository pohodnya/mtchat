# Plan 011: Responsive/Mobile Layout

## Overview

Адаптивная верстка трёхколоночного layout'а для MTChat с поддержкой мобильных устройств, изменяемой шириной колонок и унифицированными хедерами.

## Current State

Текущая реализация в `mtchat-vue/src/components/MTChat.vue`:
- Sidebar: фиксированная ширина 280px
- Main chat: flex: 1
- Info panel: фиксированная ширина 300px
- **Никакой адаптивности** — нет media queries
- Кнопка "Отправить" текстовая

## Requirements

### 1. Responsive Breakpoints

| Breakpoint | Ширина | Поведение |
|------------|--------|-----------|
| Mobile | < 768px | Одна колонка, оверлей |
| Tablet | 768px - 1199px | Две колонки |
| Desktop | ≥ 1200px | Три колонки |

### 2. Mobile Layout (< 768px)

- **Изначально**: показываем список чатов (колонка 1)
- **После выбора чата**: чат перекрывает список
  - В хедере чата появляется кнопка "← назад" (шеврон)
  - Клик возвращает к списку
- **Открытие инфо**: панель инфо перекрывает чат
  - Закрытие на крестик (уже есть)
- Все переходы — CSS transitions

### 3. Tablet Layout (768px - 1199px)

- **По умолчанию**: список + чат
- **При открытии инфо**: чат + инфо (список скрывается)
- Кнопка назад в хедере чата когда список скрыт

### 4. Desktop Layout (≥ 1200px)

- Все три колонки одновременно
- Изменение ширины drag'ом

### 5. Column Resizing (Desktop)

- Divider между колонками (4px, cursor: col-resize)
- Drag для изменения ширины
- **Ограничения**:
  - Чат (main): минимум 50% ширины контейнера
  - Список и инфо: максимум 30% каждая
  - Минимальные ширины: список 200px, инфо 240px

### 6. Unified Header Height

- Одинаковая высота для всех трёх хедеров: **48px**
- Search + Tabs в sidebar → объединить в один хедер или выровнять высоту
- Линия border-bottom на одном уровне

### 7. Send Button → Icon

- Заменить текст "Отправить" на иконку бумажного самолётика
- Круглая кнопка 44x44px (как кнопка attach)

### 8. Archived Accordion on Small Height

- При высоте экрана < 600px:
  - Открытый аккордеон занимает **всю доступную высоту** (не 50%)
  - Закрытый — сворачивается в самый низ

## Design Token Architecture

Вся верстка должна использовать CSS custom properties (токены), которые позже будут заменены на токены из дизайн системы PrimeVue.

### Token Categories

```css
/* === LAYOUT TOKENS === */
--mtchat-sidebar-width: 280px;
--mtchat-sidebar-min-width: 200px;
--mtchat-sidebar-max-percent: 30;

--mtchat-info-width: 300px;
--mtchat-info-min-width: 240px;
--mtchat-info-max-percent: 30;

--mtchat-main-min-percent: 50;

--mtchat-header-height: 48px;
--mtchat-resizer-width: 4px;

/* === SPACING TOKENS === */
--mtchat-spacing-xs: 4px;
--mtchat-spacing-sm: 8px;
--mtchat-spacing-md: 12px;
--mtchat-spacing-lg: 16px;
--mtchat-spacing-xl: 24px;

/* === SIZING TOKENS === */
--mtchat-button-size: 44px;
--mtchat-icon-size: 20px;
--mtchat-avatar-size: 40px;
--mtchat-input-height: 44px;

/* === BORDER TOKENS === */
--mtchat-border-radius-sm: 4px;
--mtchat-border-radius-md: 8px;
--mtchat-border-radius-lg: 12px;
--mtchat-border-radius-full: 50%;

/* === TRANSITION TOKENS === */
--mtchat-transition-fast: 150ms ease;
--mtchat-transition-normal: 300ms ease;

/* === BREAKPOINT TOKENS (for reference, used in JS) === */
--mtchat-breakpoint-mobile: 768px;
--mtchat-breakpoint-tablet: 1200px;

/* === Z-INDEX TOKENS === */
--mtchat-z-base: 1;
--mtchat-z-overlay: 10;
--mtchat-z-modal: 100;

/* === COLOR TOKENS (existing, will be replaced) === */
--mtchat-bg: ...
--mtchat-bg-secondary: ...
--mtchat-bg-hover: ...
--mtchat-text: ...
--mtchat-text-secondary: ...
--mtchat-border: ...
--mtchat-primary: ...
--mtchat-primary-hover: ...
```

## Implementation Plan

### Phase 1: Token-Based CSS Architecture

**File:** `mtchat-vue/src/components/MTChat.vue`

1. Добавить все токены в корневой элемент `.mtchat`
2. Заменить все hardcoded значения на токены
3. Унифицировать высоту хедеров:
- `.mtchat__header`, `.mtchat__search + .mtchat__tabs`, `.chat-info-panel__header`
- Все: `height: var(--mtchat-header-height)`

### Phase 2: Responsive State Management

**Добавить в component:**

```typescript
// Breakpoint detection
const windowWidth = ref(window.innerWidth)
const windowHeight = ref(window.innerHeight)

const isMobile = computed(() => windowWidth.value < 768)
const isTablet = computed(() => windowWidth.value >= 768 && windowWidth.value < 1200)
const isDesktop = computed(() => windowWidth.value >= 1200)

// View state for mobile/tablet
const mobileView = ref<'list' | 'chat' | 'info'>('list')
```

### Phase 3: Mobile Layout Implementation

1. **CSS Media Queries:**
```css
@media (max-width: 767px) {
  .mtchat {
    position: relative;
  }
  .mtchat__sidebar,
  .mtchat__main,
  .mtchat__info-panel {
    position: absolute;
    inset: 0;
    width: 100% !important;
  }
  /* Show/hide based on mobileView */
}
```

2. **Back Button в хедере чата:**
```html
<button v-if="isMobile || (isTablet && showInfoPanel)"
        class="mtchat__back-btn"
        @click="goBack">
  <svg><!-- chevron left --></svg>
</button>
```

3. **Transition animations:**
```css
.mtchat__sidebar,
.mtchat__main,
.mtchat__info-panel {
  transition: transform 0.3s ease;
}
```

### Phase 4: Tablet Layout

1. Показывать sidebar + main по умолчанию
2. При showInfoPanel — скрывать sidebar, показывать main + info
3. Кнопка назад в main header когда sidebar скрыт

### Phase 5: Column Resizer Component

**Inline в MTChat.vue:**

```vue
<template>
  <div class="mtchat__resizer"
       @mousedown="startResize('sidebar')"
       @touchstart="startResize('sidebar')">
  </div>
</template>
```

**Логика:**
- mousedown/touchstart → начать resize
- mousemove/touchmove → обновить ширину
- mouseup/touchend → завершить
- Валидация min/max constraints

### Phase 6: Send Button Icon

1. Заменить текст на SVG иконку (paper airplane)
2. Сделать кнопку круглой:
```css
.mtchat__btn--send {
  width: 44px;
  height: 44px;
  border-radius: 50%;
  padding: 0;
  display: flex;
  align-items: center;
  justify-content: center;
}
```

### Phase 7: Archived Height Behavior

```css
@media (max-height: 599px) {
  .mtchat__archived-section--open {
    flex: 1;
    max-height: none;
    min-height: auto;
  }
}
```

## Files to Modify

| File | Changes |
|------|---------|
| `mtchat-vue/src/components/MTChat.vue` | Token system, responsive logic, CSS, state management, resizer |
| `mtchat-vue/src/components/chat/ChatInfoPanel.vue` | Header height unification (use token) |
| `mtchat-vue/src/i18n/messages.ts` | Add "back" button translation |

## Token Migration Notes

При интеграции токенов из PrimeVue дизайн системы:
1. Создать файл `tokens.css` или использовать CSS-in-JS
2. Маппинг PrimeVue токенов на `--mtchat-*` токены
3. Возможно использование `@layer` для приоритетов

## CSS Classes Structure

```
.mtchat
  .mtchat--mobile      (< 768px)
  .mtchat--tablet      (768-1199px)
  .mtchat--desktop     (≥ 1200px)

  .mtchat--view-list   (mobile: show list)
  .mtchat--view-chat   (mobile: show chat)
  .mtchat--view-info   (mobile: show info)

  .mtchat__sidebar
  .mtchat__resizer     (between sidebar and main)
  .mtchat__main
  .mtchat__resizer     (between main and info)
  .mtchat__info-panel
```

## Verification

1. **Mobile (< 768px)**:
   - [ ] Список чатов отображается изначально
   - [ ] Клик по чату → переход к чату с анимацией
   - [ ] Кнопка назад в хедере → возврат к списку
   - [ ] Открытие инфо → перекрывает чат
   - [ ] Крестик закрывает инфо → возврат к чату

2. **Tablet (768-1199px)**:
   - [ ] Список + чат по умолчанию
   - [ ] Открытие инфо → скрывает список, показывает инфо
   - [ ] Кнопка назад → возврат к списку + чат

3. **Desktop (≥ 1200px)**:
   - [ ] Все 3 колонки видны
   - [ ] Drag resizer работает
   - [ ] Соблюдаются ограничения min/max ширины

4. **Общее**:
   - [ ] Хедеры одной высоты (48px)
   - [ ] Кнопка отправки — иконка самолётика
   - [ ] Аккордеон архива на маленькой высоте → полная высота

5. **Тестирование**:
   - Chrome DevTools device emulation
   - Реальные устройства iOS/Android
   - Resize браузера
