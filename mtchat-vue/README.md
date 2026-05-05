# @mtchat/vue

Embeddable Vue 3 chat SDK for MTChat.

Use `@mtchat/vue` when you want a ready-made chat UI, reactive composables, and low-level SDK access from a single package.

## Installation

```bash
npm install @mtchat/vue
```

Peer dependencies:

- `vue` `^3.4.0`

## Quick Start

```vue
<script setup lang="ts">
import { MTChat, type MTChatConfig } from '@mtchat/vue'

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
  locale: 'en',
}
</script>

<template>
  <div style="height: 600px;">
    <MTChat :config="config" mode="full" />
  </div>
</template>
```

Styles are injected by the package bundle. No separate `@mtchat/vue/style.css` import is required.

## Required Config

`MTChatConfig` requires:

- `baseUrl` - MTChat backend URL
- `userId` - current user identifier
- `scopeConfig` - access scopes used for "Available" dialogs
- `userProfile` - display profile used when the user joins dialogs

Optional fields include `token`, `locale`, `wsUrl`, reconnect settings, and WebSocket callbacks.

## Authentication Model

The host application is responsible for authentication and identity.

- If your backend enables JWT auth, pass `token` in `MTChatConfig`.
- If JWT auth is disabled, the SDK still requires `userId` and scope data from the host app.
- The SDK does not create users, tenants, or sessions for you.

## Main Usage Patterns

### Component

```vue
<template>
  <MTChat
    :config="config"
    mode="inline"
    :object-id="order.id"
    object-type="order"
    @message-sent="handleMessageSent"
  />
</template>
```

### Composable

```ts
import { useChat } from '@mtchat/vue'

const chat = useChat({
  config,
  autoConnect: true,
  objectId: order.id,
  objectType: 'order',
})

await chat.loadParticipatingDialogs()
```

### SDK Client

```ts
import { MTChatClient } from '@mtchat/vue'

const client = new MTChatClient(config)

client.connect()

const dialogs = await client.api.getDialogs()
const messages = await client.api.getMessages(dialogs[0].id, { limit: 50 })

client.on('message.new', (event) => {
  console.log('new message', event)
})
```

## Package Contents

- `MTChat` - ready-to-use chat component
- `useChat`, `useFileUpload` - Vue composables
- `MTChatClient`, `MTChatApi`, `MTChatWebSocket` - lower-level SDK
- exported types for dialogs, messages, attachments, events, and config

## PrimeVue

If your app uses PrimeVue, install the companion wrapper:

```bash
npm install @mtchat/vue @mtchat/vue-primevue primevue
```

See the PrimeVue integration guide in [`docs/sdk/primevue.md`](../docs/sdk/primevue.md).

## Documentation

- SDK installation: [`docs/sdk/installation.md`](../docs/sdk/installation.md)
- SDK configuration: [`docs/sdk/configuration.md`](../docs/sdk/configuration.md)
- Full mode: [`docs/sdk/full-mode.md`](../docs/sdk/full-mode.md)
- Inline mode: [`docs/sdk/inline-mode.md`](../docs/sdk/inline-mode.md)

## Publishing Checklist

Before publishing:

- `npm run typecheck`
- `npm test`
- `npm run build`
- `npm pack --dry-run`

## License

MIT
