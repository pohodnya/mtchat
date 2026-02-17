# Configuration

## MTChatConfig

The `MTChatConfig` interface defines all SDK settings. Pass it as the `config` prop to the `<MTChat>` component or to the `useChat` composable.

```typescript
import type { MTChatConfig } from '@mtchat/vue'

const config: MTChatConfig = {
  // Required
  baseUrl: 'https://chat.example.com',
  userId: 'user-uuid',
  scopeConfig: {
    tenant_uid: 'tenant-uuid',
    scope_level1: ['logistics'],
    scope_level2: ['manager'],
  },
  userProfile: {
    displayName: 'John Doe',
    company: 'Acme Corp',
  },

  // Optional
  locale: 'en',
  wsUrl: 'wss://chat.example.com/api/v1/ws',
  reconnect: true,
  reconnectInterval: 3000,
  heartbeatInterval: 30000,
  onConnect: () => console.log('Connected'),
  onDisconnect: () => console.log('Disconnected'),
  onError: (err) => console.error(err),
  onMessage: (event) => console.log('WS event:', event.type),
}
```

### Reference

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `baseUrl` | `string` | **required** | MTChat API base URL |
| `userId` | `string` | **required** | Current user's UUID |
| `scopeConfig` | `ScopeConfig` | **required** | Access control scopes |
| `userProfile` | `UserProfile` | **required** | User display profile |
| `wsUrl` | `string` | derived from `baseUrl` | WebSocket endpoint URL |
| `locale` | `'ru' \| 'en' \| 'zh'` | `'ru'` | UI language |
| `reconnect` | `boolean` | `true` | Auto-reconnect on disconnect |
| `reconnectInterval` | `number` | `3000` | Reconnect delay in ms |
| `heartbeatInterval` | `number` | `30000` | WebSocket ping interval in ms |
| `onConnect` | `() => void` | -- | Called when WebSocket connects |
| `onDisconnect` | `() => void` | -- | Called when WebSocket disconnects |
| `onError` | `(error: Error) => void` | -- | Called on errors |
| `onMessage` | `(event: WsEvent) => void` | -- | Called on every WebSocket event |

## ScopeConfig

Scopes determine which "Available" dialogs a user can see and join. The matching logic is: **tenant matches AND at least one scope_level1 matches AND at least one scope_level2 matches**.

```typescript
interface ScopeConfig {
  /** Tenant/organization ID */
  tenant_uid: string

  /** First scope level (e.g., departments) */
  scope_level1: string[]

  /** Second scope level (e.g., roles or permissions) */
  scope_level2: string[]
}
```

### Example

```typescript
// User scopes
const scopeConfig: ScopeConfig = {
  tenant_uid: 'tenant-abc',
  scope_level1: ['logistics', 'sales'],     // departments
  scope_level2: ['manager', 'viewer'],       // roles
}

// A dialog with access scope:
// { tenant_uid: 'tenant-abc', scope_level1: ['logistics'], scope_level2: ['manager', 'admin'] }
// -> Match: tenant matches, 'logistics' in scope_level1, 'manager' in scope_level2
// -> User sees this dialog in "Available" tab
```

!!! tip "Empty Scope Arrays"
    Empty arrays on the **dialog** side act as wildcards -- they match any user value. Empty arrays on the **user** side match nothing.

## UserProfile

Profile information displayed in chats when the user participates. Users can customize their visible name and contact details when joining each dialog.

```typescript
interface UserProfile {
  /** Display name (full name from user profile) */
  displayName: string

  /** Company or organization name */
  company: string

  /** Contact email (optional, can be hidden per-dialog) */
  email?: string

  /** Contact phone (optional, can be hidden per-dialog) */
  phone?: string
}
```

## WebSocket URL

By default, the WebSocket URL is derived from `baseUrl`:

- `https://chat.example.com` becomes `wss://chat.example.com/api/v1/ws`
- `http://localhost:8080` becomes `ws://localhost:8080/api/v1/ws`

Override with `wsUrl` if your WebSocket endpoint differs from the API URL (e.g., behind a reverse proxy with different routing).

## Reactive Configuration

The config is reactive. Wrap it in `computed()` to respond to changes at runtime:

```vue
<script setup lang="ts">
import { computed, ref } from 'vue'
import type { MTChatConfig } from '@mtchat/vue'

const locale = ref<'ru' | 'en' | 'zh'>('en')

const config = computed<MTChatConfig>(() => ({
  baseUrl: 'https://chat.example.com',
  userId: user.id,
  scopeConfig: user.scopes,
  userProfile: user.profile,
  locale: locale.value,
}))
</script>

<template>
  <MTChat :config="config" />
</template>
```

!!! warning "Changing `userId` or `baseUrl`"
    Changing `userId` or `baseUrl` triggers a full reconnect. The SDK will disconnect, clear state, and reconnect with the new identity. Avoid frequent changes to these properties.
