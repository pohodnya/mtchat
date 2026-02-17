# Theming

MTChat supports light and dark themes out of the box. Customize the appearance using CSS variables.

## Built-in Themes

Set the `theme` prop:

```vue
<!-- Light theme (default) -->
<MTChat :config="config" theme="light" />

<!-- Dark theme -->
<MTChat :config="config" theme="dark" />
```

The theme is applied as a CSS class (`mtchat--light` or `mtchat--dark`) on the root element.

## CSS Variables

All CSS variables use the `--mtchat-` prefix. Override them to match your application's design system.

### Color Variables

| Variable | Light | Dark | Description |
|----------|-------|------|-------------|
| `--mtchat-bg` | `#ffffff` | `#1e1e1e` | Main background |
| `--mtchat-bg-secondary` | `#f8fafc` | `#2d2d2d` | Secondary background (sidebar, panels) |
| `--mtchat-bg-hover` | `#f1f5f9` | `#3d3d3d` | Hover state background |
| `--mtchat-text` | `#1e293b` | `#e4e4e7` | Primary text color |
| `--mtchat-text-secondary` | `#64748b` | `#a1a1aa` | Secondary text (timestamps, hints) |
| `--mtchat-border` | `#e2e8f0` | `#3f3f46` | Border color |
| `--mtchat-primary` | `#3b82f6` | `#60a5fa` | Primary accent (links, buttons, badges) |
| `--mtchat-primary-bg` | `rgba(59,130,246,0.1)` | `rgba(96,165,250,0.15)` | Primary background tint |
| `--mtchat-danger` | `#ef4444` | `#f87171` | Danger/error color |
| `--mtchat-success` | `#22c55e` | `#4ade80` | Success/online indicator |
| `--mtchat-warning` | `#f59e0b` | `#fbbf24` | Warning color |

### Layout Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `--mtchat-header-height` | `48px` | Header bar height |
| `--mtchat-resizer-width` | `2px` | Sidebar resize handle width |
| `--mtchat-spacing-xs` | `4px` | Extra small spacing |
| `--mtchat-spacing-sm` | `8px` | Small spacing |
| `--mtchat-spacing-md` | `12px` | Medium spacing |
| `--mtchat-spacing-lg` | `16px` | Large spacing |

## Custom Theme

Create a custom theme by defining a CSS class with overridden variables:

```css
/* custom-theme.css */
.mtchat--brand {
  --mtchat-bg: #fafafa;
  --mtchat-bg-secondary: #f0f0f0;
  --mtchat-bg-hover: #e8e8e8;
  --mtchat-text: #222222;
  --mtchat-text-secondary: #666666;
  --mtchat-border: #d0d0d0;
  --mtchat-primary: #6366f1;
  --mtchat-primary-bg: rgba(99, 102, 241, 0.1);
  --mtchat-danger: #dc2626;
  --mtchat-success: #16a34a;
  --mtchat-warning: #d97706;
}
```

Then use the custom theme name:

```vue
<MTChat :config="config" theme="brand" />
```

## Integrating with Your Design System

Map MTChat variables to your application's existing design tokens:

```vue
<template>
  <div class="app-chat-wrapper">
    <MTChat :config="config" theme="light" />
  </div>
</template>

<style>
.app-chat-wrapper .mtchat--light {
  --mtchat-primary: var(--app-brand-color);
  --mtchat-bg: var(--app-surface);
  --mtchat-text: var(--app-text);
  --mtchat-border: var(--app-divider);
}
</style>
```

## Dynamic Theme Switching

Bind the theme prop to a reactive value:

```vue
<script setup lang="ts">
import { ref, computed } from 'vue'

const isDark = ref(false)
const theme = computed(() => isDark.value ? 'dark' : 'light')
</script>

<template>
  <button @click="isDark = !isDark">Toggle Theme</button>
  <MTChat :config="config" :theme="theme" />
</template>
```

Theme changes are instant -- no component remount required.

## Scoped Styling

MTChat styles are scoped under the `.mtchat` root class. They do not leak into your application and your application styles do not affect the chat. If you need to override specific internal styles, target them through the `.mtchat` prefix:

```css
/* Override message font size */
.mtchat .mtchat__message-content {
  font-size: 15px;
}

/* Override sidebar width */
.mtchat .mtchat__sidebar {
  width: 320px;
}
```

!!! warning "Internal CSS Classes"
    Internal class names (prefixed with `mtchat__`) are not part of the public API and may change between versions. Prefer using CSS variables for customization.
