# @mtchat/vue-primevue

PrimeVue integration layer for MTChat.

Use `@mtchat/vue-primevue` when your application already uses PrimeVue 4.x and you want MTChat to reuse PrimeVue primitives and theme tokens instead of the default SDK UI primitives.

## Installation

```bash
npm install @mtchat/vue @mtchat/vue-primevue primevue
```

Peer dependencies:

- `vue` `^3.4.0`
- `primevue` `^4.0.0`
- `@mtchat/vue` `^0.4.0`

## Quick Start

```vue
<script setup lang="ts">
import { MTChatPrime, type MTChatConfig } from '@mtchat/vue-primevue'

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
    <MTChatPrime :config="config" theme="light" />
  </div>
</template>
```

`MTChatPrime` uses the same config, props, events, composables, and SDK classes as `@mtchat/vue`.

## What This Package Adds

- `MTChatPrime` - preconfigured component with PrimeVue-backed registry
- `primevueRegistry` - registry for manual setup with base `MTChat`
- PrimeVue adapter primitives such as `PrimeButton`, `PrimeDialog`, `PrimeInput`
- re-exports of types, composables, and SDK classes from `@mtchat/vue`

## Manual Registry Setup

If you want to keep using the base `MTChat` component, provide the PrimeVue registry manually:

```vue
<script setup lang="ts">
import { MTChat, provideRegistry } from '@mtchat/vue'
import { primevueRegistry } from '@mtchat/vue-primevue'
import '@mtchat/vue-primevue/theme/aura.css'

provideRegistry(primevueRegistry)
</script>

<template>
  <div style="height: 600px;">
    <MTChat :config="config" />
  </div>
</template>
```

Use the manual setup path when you want to mix PrimeVue primitives with custom ones.

## Theme Notes

- `MTChatPrime` already imports the bundled PrimeVue token mapping theme.
- For manual registry usage, import `@mtchat/vue-primevue/theme/aura.css`.
- You can override MTChat variables on `.mtchat-prime` and PrimeVue tokens through your normal PrimeVue theme setup.

## Documentation

- Installation: [`../docs/sdk/installation.md`](../docs/sdk/installation.md)
- PrimeVue guide: [`../docs/sdk/primevue.md`](../docs/sdk/primevue.md)
- Base SDK config: [`../docs/sdk/configuration.md`](../docs/sdk/configuration.md)

## Publishing Checklist

Before publishing:

- `npm run typecheck`
- `npm run build`
- `npm pack --dry-run`

## License

MIT
