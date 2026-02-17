# Темы

MTChat поддерживает светлую и тёмную темы из коробки с возможностью настройки через CSS-переменные.

## Встроенные темы

```vue
<!-- Светлая (по умолчанию) -->
<MTChat :config="config" theme="light" />

<!-- Тёмная -->
<MTChat :config="config" theme="dark" />
```

Тема применяется как CSS-класс (`mtchat--light` или `mtchat--dark`) на корневом элементе.

## CSS-переменные

### Цвета

| Переменная | Светлая | Тёмная | Описание |
|------------|---------|--------|----------|
| `--mtchat-bg` | `#ffffff` | `#1e1e1e` | Основной фон |
| `--mtchat-bg-secondary` | `#f8fafc` | `#2d2d2d` | Вторичный фон |
| `--mtchat-bg-hover` | `#f1f5f9` | `#3d3d3d` | Фон при наведении |
| `--mtchat-text` | `#1e293b` | `#e4e4e7` | Основной текст |
| `--mtchat-text-secondary` | `#64748b` | `#a1a1aa` | Вторичный текст |
| `--mtchat-border` | `#e2e8f0` | `#3f3f46` | Границы |
| `--mtchat-primary` | `#3b82f6` | `#60a5fa` | Акцент (ссылки, кнопки) |
| `--mtchat-danger` | `#ef4444` | `#f87171` | Ошибка/опасность |
| `--mtchat-success` | `#22c55e` | `#4ade80` | Успех/онлайн |
| `--mtchat-warning` | `#f59e0b` | `#fbbf24` | Предупреждение |

## Кастомная тема

```css
.mtchat--brand {
  --mtchat-bg: #fafafa;
  --mtchat-primary: #6366f1;
  --mtchat-text: #222222;
  --mtchat-border: #d0d0d0;
}
```

```vue
<MTChat :config="config" theme="brand" />
```

## Интеграция с вашей темой

```vue
<style>
.my-app .mtchat--light {
  --mtchat-primary: var(--app-brand-color);
  --mtchat-bg: var(--app-surface);
  --mtchat-text: var(--app-text);
}
</style>
```

## Динамическое переключение

```vue
<script setup>
import { ref, computed } from 'vue'
const isDark = ref(false)
const theme = computed(() => isDark.value ? 'dark' : 'light')
</script>

<template>
  <button @click="isDark = !isDark">Переключить тему</button>
  <MTChat :config="config" :theme="theme" />
</template>
```

Переключение тем происходит мгновенно -- без перезагрузки компонента и потери состояния.
