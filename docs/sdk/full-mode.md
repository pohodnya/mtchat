# Full Mode

Full mode displays a sidebar with dialog lists alongside the chat area. This is the default mode, suitable for dedicated chat pages or panels.

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
  <div style="height: 600px;">
    <MTChat :config="config" mode="full" theme="light" />
  </div>
</template>
```

!!! important "Container Height"
    The `<MTChat>` component fills its parent container. You must set a height on the parent element (e.g., `height: 600px`, `height: 100vh`, or `flex: 1` in a flex layout).

## Sidebar Tabs

The sidebar contains two tabs:

- **My Chats** -- dialogs where the user is a participant, sorted with pinned chats first, then by most recent message
- **Available** -- dialogs the user can join based on scope matching

Below the active chats, an **Archived** accordion shows dialogs the user has archived. Archived chats load lazily when the accordion is opened.

## Props

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `config` | `MTChatConfig` | **required** | SDK configuration |
| `mode` | `'full' \| 'inline'` | `'full'` | Display mode |
| `dialogId` | `string` | -- | Pre-select a dialog on mount |
| `showHeader` | `boolean` | `true` | Show the chat header bar |
| `showSidebar` | `boolean` | `true` | Show the dialog list sidebar |
| `theme` | `string` | `'light'` | Theme name (`'light'` or `'dark'`) |

## Events

| Event | Payload | Description |
|-------|---------|-------------|
| `connected` | -- | WebSocket connection established |
| `disconnected` | -- | WebSocket connection lost |
| `error` | `Error` | An error occurred |
| `message-sent` | `Message` | User sent a message |
| `dialog-selected` | `DialogListItem` | User selected a dialog from the list |
| `dialog-joined` | `string` | User joined a dialog (dialog ID) |
| `dialog-left` | `string` | User left a dialog (dialog ID) |

### Event Examples

```vue
<script setup lang="ts">
import type { Message, DialogListItem } from '@mtchat/vue'

function onMessageSent(message: Message) {
  console.log('Message sent:', message.id, message.content)
}

function onDialogSelected(dialog: DialogListItem) {
  // Update URL, breadcrumb, etc.
  router.push({ query: { chat: dialog.id } })
}

function onDialogJoined(dialogId: string) {
  console.log('User joined dialog:', dialogId)
}
</script>

<template>
  <MTChat
    :config="config"
    @message-sent="onMessageSent"
    @dialog-selected="onDialogSelected"
    @dialog-joined="onDialogJoined"
  />
</template>
```

## Slots

### `sidebar-action`

Add a custom action button to the sidebar header (e.g., "Create Chat"):

```vue
<template>
  <MTChat :config="config">
    <template #sidebar-action>
      <button @click="createNewChat">+ New Chat</button>
    </template>
  </MTChat>
</template>
```

### `header-menu-actions`

Add custom items to the chat header dropdown menu. Items appear before the built-in "Leave Chat" action:

```vue
<template>
  <MTChat :config="config">
    <template #header-menu-actions="{ dialog, closeMenu, menuItemClass }">
      <button
        :class="menuItemClass"
        @click="openInNewTab(dialog); closeMenu()"
      >
        Open in new tab
      </button>
    </template>
  </MTChat>
</template>
```

The slot provides:

| Property | Type | Description |
|----------|------|-------------|
| `dialog` | `DialogListItem` | Current dialog |
| `closeMenu` | `() => void` | Closes the dropdown menu |
| `menuItemClass` | `string` | CSS class for consistent menu item styling |

## Features

### Dialog Search

Press ++cmd+k++ (Mac) or ++ctrl+k++ (Windows/Linux) to focus the search input. Type to filter dialogs by title or participant company name. Press ++esc++ to clear.

### Context Menu

Right-click a dialog in the sidebar to access:

- Pin / Unpin
- Archive / Unarchive
- Mute / Unmute notifications

### Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| ++enter++ | Send message |
| ++shift+enter++ | New line |
| ++arrow-up++ | Edit last message (when input is empty) |
| ++esc++ | Cancel edit/reply, close panels |
| ++cmd+k++ / ++ctrl+k++ | Focus search |
| ++cmd+b++ | Bold |
| ++cmd+i++ | Italic |
| ++cmd+u++ | Underline |
| ++cmd+k++ | Insert/edit link (in editor) |

### Resizable Sidebar

In desktop viewports, drag the sidebar edge to resize it. The sidebar collapses automatically on mobile, showing a back button to return to the chat list.

## Responsive Layout

The component adapts to different screen sizes:

- **Desktop** (> 1024px): sidebar + chat area side by side
- **Tablet** (768--1024px): sidebar overlays or toggles
- **Mobile** (< 768px): full-screen views with navigation between sidebar and chat
