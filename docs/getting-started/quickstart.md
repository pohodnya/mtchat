# Quick Start

Get MTChat running in under 5 minutes.

## Prerequisites

- Docker and Docker Compose
- Node.js 18+ (for the Vue SDK)

## 1. Start the Backend

```bash
git clone https://github.com/nicenemo/mtchat.git
cd mtchat
docker-compose up -d
```

Verify the API is running:

```bash
curl http://localhost:8080/health
# {"status":"ok"}
```

## 2. Create a Chat via Management API

Use the Management API from your backend to create a dialog:

```bash
curl -X POST http://localhost:8080/api/v1/management/dialogs \
  -H "Authorization: Bearer demo-admin-token" \
  -H "Content-Type: application/json" \
  -d '{
    "object_id": "550e8400-e29b-41d4-a716-446655440000",
    "object_type": "order",
    "title": "Order #1234 Discussion",
    "participants": [
      {
        "user_id": "11111111-1111-1111-1111-111111111111",
        "display_name": "Alice",
        "company": "Acme Inc",
        "joined_as": "creator"
      }
    ],
    "access_scopes": [
      {
        "tenant_uid": "tenant-uuid",
        "scope_level1": ["logistics"],
        "scope_level2": ["manager", "admin"]
      }
    ]
  }'
```

## 3. Install the Vue SDK

```bash
npm install @mtchat/vue
```

## 4. Add MTChat to Your App

### Full Mode (Chat List + Chat Area)

```vue
<template>
  <MTChat :config="chatConfig" mode="full" theme="light" />
</template>

<script setup lang="ts">
import { MTChat } from '@mtchat/vue'
import '@mtchat/vue/style.css'

const chatConfig = {
  baseUrl: 'http://localhost:8080',
  userId: '11111111-1111-1111-1111-111111111111',
  scopeConfig: {
    tenant_uid: 'tenant-uuid',
    scope_level1: ['logistics'],
    scope_level2: ['manager'],
  },
  userProfile: {
    displayName: 'Alice',
    company: 'Acme Inc',
  },
}
</script>
```

### Inline Mode (Single Chat on an Object Page)

```vue
<template>
  <div class="order-page">
    <h1>Order #1234</h1>

    <MTChat
      :config="chatConfig"
      mode="inline"
      object-id="550e8400-e29b-41d4-a716-446655440000"
      object-type="order"
      theme="light"
    />
  </div>
</template>
```

## 5. Open the Demo App

Visit [http://localhost](http://localhost) to see the full demo application with:

- Admin panel for managing tenants, users, objects, and dialogs
- Full mode: chat list with "My Chats" and "Available" tabs
- Inline mode: TMS-style layout with a data table and an embedded chat
- User switching to test multi-user scenarios

## Next Steps

- [Architecture](../guide/architecture.md) -- understand how MTChat works
- [Management API](../api/management.md) -- create dialogs and manage participants from your backend
- [Chat API](../api/chat.md) -- endpoint reference for the frontend SDK
- [Vue SDK Components](../sdk/components.md) -- props, events, and slots
- [Theming](../sdk/theming.md) -- customize the look and feel
- [Configuration](../configuration.md) -- all backend environment variables
