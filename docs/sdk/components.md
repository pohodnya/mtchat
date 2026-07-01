# Vue SDK Overview

The MTChat Vue SDK provides a ready-to-use chat component for Vue 3 applications. It connects to the MTChat backend and handles real-time messaging, file uploads, participant management, and more.

## Packages

| Package | Description |
|---------|-------------|
| `@mtchat/vue` | Core SDK with built-in UI primitives |
| `@mtchat/vue-primevue` | PrimeVue integration (optional) |

## Display Modes

### Full Mode

A complete chat interface with a sidebar dialog list and chat area. Suitable for dedicated chat pages.

```vue
<div style="height: 600px;">
  <MTChat :config="config" mode="full" />
</div>
```

See [Full Mode](full-mode.md) for details.

### Inline Mode

A single chat bound to a business object, designed for embedding inside detail pages.

```vue
<MTChat
  :config="config"
  mode="inline"
  :object-id="order.id"
  object-type="order"
/>
```

See [Inline Mode](inline-mode.md) for details.

## Features

- **Rich text editing** -- bold, italic, underline, links, lists, quotes, code blocks, @mentions
- **File attachments** -- drag & drop, 40+ file types, built-in image/PDF viewer
- **Real-time messaging** -- WebSocket with auto-reconnect and heartbeat
- **Message actions** -- reply, edit, delete, with keyboard shortcuts
- **Dialog management** -- search, archive, pin, mute notifications
- **User presence** -- online/offline indicators
- **Read receipts** -- checkmarks with reader details
- **Unread tracking** -- badge counts, "new messages" divider, auto-mark-as-read
- **Infinite scroll** -- bidirectional loading with jump-to-message
- **Responsive layout** -- adapts to desktop, tablet, and mobile
- **Theming** -- light/dark modes with CSS variable customization
- **i18n** -- Russian, English, Chinese (no external dependencies)
- **Accessibility** -- ARIA attributes, keyboard navigation

## Architecture

```
@mtchat/vue
├── MTChat          Component (full UI)
├── useChat         Composable (state & actions)
├── useFileUpload   Composable (file uploads)
├── MTChatClient    SDK class (API + WebSocket)
└── Primitives      Pluggable UI components (Button, Dialog, Menu, ...)

@mtchat/vue-primevue
├── MTChatPrime     Component (MTChat + PrimeVue registry)
└── primevueRegistry  Registry mapping
```

The SDK uses a **component registry** pattern. UI primitives (buttons, inputs, dialogs, menus) can be swapped between built-in implementations and PrimeVue components, or replaced with your own.

## Props

Full prop reference for `<MTChat>` (`@mtchat/vue`). `<MTChatPrime>` (`@mtchat/vue-primevue`) accepts the same props (see the `theme` note below) -- it is a thin wrapper that forwards them to `<MTChat>` unchanged.

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `config` | `MTChatConfig` | **required** | SDK configuration |
| `mode` | `'full' \| 'inline'` | `'full'` | Display mode -- see [Full Mode](full-mode.md) / [Inline Mode](inline-mode.md) |
| `objectId` | `string` | -- | Inline mode: business object ID |
| `objectType` | `string` | -- | Inline mode: business object type (e.g. `'order'`, `'tender'`) |
| `dialogId` | `string` | -- | Full mode: pre-select a dialog on mount |
| `showHeader` | `boolean` | `true` | Show the chat header bar |
| `showSidebar` | `boolean` | `true` | Show the dialog list sidebar (full mode only; automatically disabled in inline mode) |
| `showTabs` | `boolean` | `true` | Full mode: show the tab switcher (Participating / Available). UI only -- does not affect which data is loaded |
| `showSearch` | `boolean` | `true` | Full mode: show the dialog search input. UI only |
| `searchPlaceholder` | `string` | -- | Full mode: override the search input placeholder text |
| `showContextMenu` | `boolean` | `true` | Full mode: enable the right-click context menu on dialog items |
| `theme` | `string` | `'light'` | Theme name, applied as CSS class `mtchat--${theme}`. On `<MTChatPrime>` this is narrowed to `'light' \| 'dark' \| undefined` (`undefined` auto-detects from the PrimeVue dark mode selector) -- see [PrimeVue](primevue.md) |
| `token` | `string` | -- | JWT token for authentication, overrides `config.token` |
| `interceptObjectNavigation` | `boolean` | `false` | Replace the default `<a href>` navigation to the dialog's `object_url` with a client-side `object-navigate` emit, so the host app can handle navigation itself (e.g. via `vue-router`) instead of a full-page browser navigation. See [Events](events.md#component-events) |

## Quick Links

| Topic | Description |
|-------|-------------|
| [Installation](installation.md) | Setup and dependencies |
| [Configuration](configuration.md) | MTChatConfig reference |
| [Full Mode](full-mode.md) | Chat list interface |
| [Inline Mode](inline-mode.md) | Embedded single-chat |
| [Theming](theming.md) | CSS variables and custom themes |
| [i18n](i18n.md) | Language configuration |
| [PrimeVue](primevue.md) | PrimeVue integration |
| [Events & Composables](events.md) | Events, slots, headless usage |
