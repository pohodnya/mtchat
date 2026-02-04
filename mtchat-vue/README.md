# @mtchat/vue

Vue.js SDK for MultitenancyChat - embeddable chat component and TypeScript API client.

## Installation

```bash
npm install @mtchat/vue
```

## Quick Start

### Using the Chat Component

```vue
<template>
  <MTChat
    :config="chatConfig"
    :show-header="true"
    :show-sidebar="true"
    theme="light"
    @connected="onConnected"
    @message-sent="onMessageSent"
  />
</template>

<script setup>
import { MTChat } from '@mtchat/vue'
import '@mtchat/vue/style.css'

const chatConfig = {
  baseUrl: 'http://localhost:8080',
  employeeId: 'employee-uuid',
}

const onConnected = () => console.log('Connected!')
const onMessageSent = (msg) => console.log('Sent:', msg)
</script>
```

### Using the SDK Directly

```typescript
import { MTChatClient } from '@mtchat/vue'

const client = new MTChatClient({
  baseUrl: 'http://localhost:8080',
  employeeId: 'employee-uuid',
  onMessage: (event) => console.log('Event:', event),
})

// Connect to WebSocket
client.connect()

// Use REST API
const dialogs = await client.api.getDialogs()
const messages = await client.api.getMessages(dialogId)

// Subscribe to dialog updates
client.subscribe(dialogId)

// Send a message
await client.sendMessage(dialogId, 'Hello!')

// Disconnect
client.disconnect()
```

### Using the Composable

```vue
<script setup>
import { useChat } from '@mtchat/vue'

const {
  messages,
  dialogs,
  isConnected,
  sendMessage,
  loadMessages,
  selectDialog,
} = useChat({
  baseUrl: 'http://localhost:8080',
  employeeId: 'employee-uuid',
})

// Send a message
await sendMessage('Hello!')

// Select a dialog
await selectDialog(dialogId)
</script>
```

## API Reference

### MTChatClient

Main SDK client combining REST API and WebSocket.

```typescript
const client = new MTChatClient({
  baseUrl: string,           // Backend URL
  employeeId: string,        // Current employee UUID
  wsUrl?: string,            // WebSocket URL (auto-derived if not set)
  onConnect?: () => void,
  onDisconnect?: () => void,
  onError?: (error: Error) => void,
  onMessage?: (event: WsEvent) => void,
  reconnect?: boolean,       // Auto-reconnect (default: true)
  reconnectInterval?: number, // Reconnect delay ms (default: 3000)
  heartbeatInterval?: number, // Ping interval ms (default: 30000)
})

// Methods
client.connect()
client.disconnect()
client.subscribe(dialogId)
client.unsubscribe(dialogId)
client.on(event, handler)    // Returns unsubscribe function
client.off(event, handler)
await client.sendMessage(dialogId, content)

// Properties
client.api                   // MTChatApi instance
client.isConnected          // boolean
client.employeeId           // string
```

### MTChatApi

REST API client.

```typescript
const api = new MTChatApi(baseUrl)

// Tenants
await api.getTenants()
await api.getTenant(id)
await api.createTenant(name, externalId)

// Employees
await api.getEmployees(tenantId?)
await api.getEmployee(id)
await api.getEmployeeByExternalId(externalId)
await api.createEmployee(tenantId, externalId, fullName, hasAccessToAnyDialog?)

// Dialogs
await api.getDialogs(tenantId?)
await api.getDialog(id)
await api.createDialog(tenantAId, tenantBId, contextId?)

// Messages
await api.getMessages(dialogId, { limit?, after?, before? })
await api.getMessage(dialogId, messageId)
await api.sendMessage(dialogId, senderId, content)
await api.editMessage(dialogId, messageId, content)
await api.deleteMessage(dialogId, messageId)

// Health
await api.healthCheck()
```

### MTChat Component Props

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| config | MTChatConfig | required | SDK configuration |
| dialogId | string | - | Initial dialog to open |
| showHeader | boolean | true | Show chat header |
| showSidebar | boolean | true | Show dialogs sidebar |
| theme | 'light' \| 'dark' | 'light' | Color theme |

### MTChat Component Events

| Event | Payload | Description |
|-------|---------|-------------|
| connected | - | WebSocket connected |
| disconnected | - | WebSocket disconnected |
| error | Error | Error occurred |
| message-sent | Message | Message was sent |
| dialog-selected | Dialog | Dialog was selected |

## Types

```typescript
interface Tenant {
  id: string
  name: string
  external_id: string
  created_at: string
}

interface Employee {
  id: string
  tenant_id: string
  external_id: string
  full_name: string
  has_access_to_any_dialog: boolean
  created_at: string
}

interface Dialog {
  id: string
  chat_key: string
  tenant_a_id: string
  tenant_b_id: string
  context_id?: string
  created_at: string
}

interface Message {
  id: string
  dialog_id: string
  sender_id: string
  content: string
  sent_at: string
  last_edited_at?: string
}
```

## License

MIT
