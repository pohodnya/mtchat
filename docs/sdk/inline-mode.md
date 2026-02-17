# Inline Mode

Inline mode displays a single chat bound to a business object. Use this to embed a chat directly inside object detail pages -- orders, tenders, routes, or any entity in your application.

## Basic Usage

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
  locale: 'en',
}
</script>

<template>
  <div class="order-detail">
    <h1>Order #{{ order.number }}</h1>
    <p>Status: {{ order.status }}</p>

    <!-- Chat embedded in the order page -->
    <div style="height: 400px;">
      <MTChat
        :config="config"
        mode="inline"
        :object-id="order.id"
        object-type="order"
        theme="light"
      />
    </div>
  </div>
</template>
```

## How It Works

1. The SDK calls `GET /api/v1/dialogs/by-object/{objectType}/{objectId}` to find the most recent dialog for the given object.
2. If a dialog exists and the user is a participant, messages load immediately.
3. If the user is a potential participant (scope match), a "Join to view messages" prompt is shown.
4. If no dialog exists, a placeholder message is displayed.

!!! note "Multiple Dialogs Per Object"
    Multiple dialogs can exist for the same object. In inline mode, the SDK loads the **most recently created** dialog. Use full mode if users need to see all dialogs for an object.

## Props

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `config` | `MTChatConfig` | **required** | SDK configuration |
| `mode` | `'inline'` | -- | Must be set to `'inline'` |
| `objectId` | `string` | **required** | Business object ID |
| `objectType` | `string` | **required** | Business object type (e.g., `'order'`, `'tender'`) |
| `showHeader` | `boolean` | `true` | Show the chat header |
| `theme` | `string` | `'light'` | Theme name |

In inline mode, `showSidebar` is automatically disabled.

## Reactive Object Binding

Change `objectId` or `objectType` to switch between objects dynamically:

```vue
<script setup lang="ts">
import { ref } from 'vue'

const selectedOrderId = ref('order-1')
</script>

<template>
  <select v-model="selectedOrderId">
    <option value="order-1">Order #1</option>
    <option value="order-2">Order #2</option>
  </select>

  <div style="height: 400px;">
    <MTChat
      :config="config"
      mode="inline"
      :object-id="selectedOrderId"
      object-type="order"
    />
  </div>
</template>
```

When the `objectId` changes, the SDK automatically disconnects from the current dialog and loads the new one.

## TMS-Style Layout Example

A typical TMS (Transport Management System) layout with a data table and an embedded chat panel:

```vue
<template>
  <div class="tms-layout">
    <!-- Left: data table -->
    <div class="tms-table">
      <table>
        <tr
          v-for="order in orders"
          :key="order.id"
          :class="{ active: order.id === selectedOrderId }"
          @click="selectedOrderId = order.id"
        >
          <td>{{ order.number }}</td>
          <td>{{ order.status }}</td>
          <td>{{ order.customer }}</td>
        </tr>
      </table>
    </div>

    <!-- Right: embedded chat -->
    <div class="tms-chat-panel">
      <MTChat
        v-if="selectedOrderId"
        :config="config"
        mode="inline"
        :object-id="selectedOrderId"
        object-type="order"
        theme="light"
      />
      <p v-else class="placeholder">Select an order to view chat</p>
    </div>
  </div>
</template>

<style>
.tms-layout {
  display: flex;
  height: 100vh;
}
.tms-table {
  flex: 1;
  overflow: auto;
}
.tms-chat-panel {
  width: 480px;
  border-left: 1px solid #e2e8f0;
}
</style>
```

## Creating Dialogs

Inline mode only displays existing dialogs. To create dialogs, use the Management API from your backend:

```bash
curl -X POST https://chat.example.com/api/v1/management/dialogs \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "object_id": "order-123",
    "object_type": "order",
    "title": "Order #123 Discussion",
    "created_by": "user-uuid",
    "participants": [
      {
        "user_id": "user-1",
        "display_name": "John Doe",
        "company": "Acme Corp"
      }
    ],
    "access_scopes": [
      {
        "tenant_uid": "tenant-abc",
        "scope_level1": ["logistics"],
        "scope_level2": ["manager", "admin"]
      }
    ]
  }'
```

See the [Management API](../api/management.md) documentation for details.
