# Installation

## Requirements

- **Vue.js** 3.4 or later
- **Node.js** 18 or later
- A running MTChat backend instance

## Install the SDK

=== "npm"

    ```bash
    npm install @mtchat/vue
    ```

=== "yarn"

    ```bash
    yarn add @mtchat/vue
    ```

=== "pnpm"

    ```bash
    pnpm add @mtchat/vue
    ```

### Peer Dependencies

The SDK requires Vue 3.4+ as a peer dependency. If not already in your project:

```bash
npm install vue@^3.4
```

## Basic Setup

Import the component and its styles in your Vue application:

```vue
<script setup lang="ts">
import { MTChat } from '@mtchat/vue'
import '@mtchat/vue/style.css'
import type { MTChatConfig } from '@mtchat/vue'

const config: MTChatConfig = {
  baseUrl: 'https://chat.example.com',
  userId: currentUser.id,
  scopeConfig: {
    tenant_uid: currentUser.tenantId,
    scope_level1: currentUser.departments,
    scope_level2: currentUser.permissions,
  },
  userProfile: {
    displayName: currentUser.name,
    company: currentUser.company,
  },
}
</script>

<template>
  <MTChat :config="config" />
</template>
```

!!! note "Style Import"
    The `@mtchat/vue/style.css` import is required. It contains all component styles. Without it, the chat UI will not render correctly.

## PrimeVue Integration

If your application uses [PrimeVue](https://primevue.org/), install the companion package for seamless UI integration:

```bash
npm install @mtchat/vue-primevue
```

This requires PrimeVue 4.x as a peer dependency:

```bash
npm install primevue@^4.0
```

See the [PrimeVue Integration](primevue.md) page for detailed setup instructions.

## Bundle Size

The SDK is built as an ES module with tree-shaking support. Typical bundle sizes:

| Import | Size (gzip) |
|--------|-------------|
| Full component (`MTChat`) | ~80 KB |
| Composables only (`useChat`) | ~15 KB |
| Types only | 0 KB |

## TypeScript Support

The SDK ships with full TypeScript declarations. No additional `@types` packages are needed. All exported types are available from the main entry point:

```typescript
import type {
  MTChatConfig,
  Message,
  DialogListItem,
  WsEvent,
  UseChatReturn,
} from '@mtchat/vue'
```

## Next Steps

- [Configuration](configuration.md) -- configure the SDK for your environment
- [Full Mode](full-mode.md) -- set up the chat list interface
- [Inline Mode](inline-mode.md) -- embed a chat in an object detail page
