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

Import the component in your Vue application:

```vue
<script setup lang="ts">
import { MTChat } from '@mtchat/vue'
import type { MTChatConfig } from '@mtchat/vue'

const config: MTChatConfig = {
  baseUrl: 'https://chat.example.com',
  userId: currentUser.id,
  scopeConfig: {
    scopeLevel0: [currentUser.tenantId],
    scopeLevel1: currentUser.departments,
    scopeLevel2: currentUser.permissions,
  },
  userProfile: {
    displayName: currentUser.name,
    company: currentUser.company,
  },
}
</script>

<template>
  <div style="height: 600px;">
    <MTChat :config="config" />
  </div>
</template>
```

!!! note "Style Import"
    `@mtchat/vue` injects its styles from the package bundle. No separate CSS import is required.

## PrimeVue Integration

If your application uses [PrimeVue](https://primevue.org/), install the companion package for seamless UI integration:

```bash
npm install @mtchat/vue @mtchat/vue-primevue primevue
```

This requires PrimeVue 4.x as a peer dependency.

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
