# Events, Slots & Composables

## Component Events

The `<MTChat>` component emits the following events:

| Event | Payload | Description |
|-------|---------|-------------|
| `connected` | -- | WebSocket connection established |
| `disconnected` | -- | WebSocket connection lost |
| `error` | `Error` | An error occurred (API or WebSocket) |
| `message-sent` | `Message` | User successfully sent a message |
| `dialog-selected` | `DialogListItem` | User selected a dialog from the sidebar |
| `dialog-joined` | `string` | User joined a dialog (emits dialog ID) |
| `dialog-left` | `string` | User left a dialog (emits dialog ID) |

### Usage

```vue
<script setup lang="ts">
import { MTChat, type Message, type DialogListItem } from '@mtchat/vue'

function onConnected() {
  console.log('Chat WebSocket connected')
}

function onError(error: Error) {
  // Report to your error tracking service
  errorTracker.captureException(error)
}

function onMessageSent(message: Message) {
  // Trigger notifications, analytics, etc.
  analytics.track('chat_message_sent', {
    dialogId: message.dialog_id,
    hasAttachments: (message.attachments?.length ?? 0) > 0,
  })
}

function onDialogSelected(dialog: DialogListItem) {
  // Sync URL with selected dialog
  router.replace({ query: { chat: dialog.id } })
}
</script>

<template>
  <MTChat
    :config="config"
    @connected="onConnected"
    @disconnected="() => showReconnectBanner = true"
    @error="onError"
    @message-sent="onMessageSent"
    @dialog-selected="onDialogSelected"
    @dialog-joined="(id) => console.log('Joined:', id)"
    @dialog-left="(id) => console.log('Left:', id)"
  />
</template>
```

## Slots

### `sidebar-action`

Adds a custom element to the sidebar header area. Commonly used for a "Create Chat" button:

```vue
<MTChat :config="config">
  <template #sidebar-action>
    <button class="create-chat-btn" @click="showCreateDialog = true">
      + New Chat
    </button>
  </template>
</MTChat>
```

### `header-menu-actions`

Inserts custom items into the chat header dropdown menu (the three-dot menu). Items appear before the built-in actions (Leave Chat, etc.):

```vue
<MTChat :config="config">
  <template #header-menu-actions="{ dialog, closeMenu, menuItemClass }">
    <button
      :class="menuItemClass"
      @click="viewObject(dialog.object_id); closeMenu()"
    >
      View Order
    </button>
    <button
      :class="menuItemClass"
      @click="shareDialog(dialog.id); closeMenu()"
    >
      Share Chat Link
    </button>
  </template>
</MTChat>
```

**Slot props:**

| Prop | Type | Description |
|------|------|-------------|
| `dialog` | `DialogListItem` | The currently selected dialog |
| `closeMenu` | `() => void` | Call to close the dropdown menu |
| `menuItemClass` | `string` | CSS class for consistent styling with built-in items |

## WebSocket Events

The SDK receives real-time events via WebSocket. Listen to raw events using the `onMessage` config callback:

```typescript
const config: MTChatConfig = {
  // ...
  onMessage: (event: WsEvent) => {
    switch (event.type) {
      case 'message.new':
        console.log('New message in dialog:', event.dialog_id)
        break
      case 'message.read':
        console.log('Messages read by:', event.user_id)
        break
      case 'presence.update':
        console.log('User', event.user_id, 'online:', event.is_online)
        break
    }
  },
}
```

### Event Types

| Event Type | Fields | Description |
|------------|--------|-------------|
| `connected` | -- | WebSocket connection opened |
| `disconnected` | -- | WebSocket connection closed |
| `message.new` | `dialog_id`, `message` | New message received |
| `message.read` | `dialog_id`, `user_id`, `last_read_message_id` | User read messages |
| `message.edited` | `dialog_id`, `message` | Message was edited |
| `message.deleted` | `dialog_id`, `message` | Message was deleted |
| `participant.joined` | `dialog_id`, `user_id` | User joined a dialog |
| `participant.left` | `dialog_id`, `user_id` | User left a dialog |
| `dialog.archived` | `dialog_id` | Dialog was archived |
| `dialog.unarchived` | `dialog_id` | Dialog was unarchived |
| `presence.update` | `user_id`, `is_online` | User online status changed |
| `pong` | -- | Heartbeat response |
| `error` | `message` | Server error |

## Composables

For custom UI or headless usage, use the composables directly instead of the `<MTChat>` component.

### `useChat`

The main composable providing all chat state and actions:

```typescript
import { useChat, type UseChatOptions } from '@mtchat/vue'

const options: UseChatOptions = {
  config: chatConfig,
  autoConnect: true,      // default: true
  dialogId: 'dialog-1',   // optional: pre-select dialog
  objectId: 'order-123',  // optional: inline mode
  objectType: 'order',    // optional: inline mode
}

const chat = useChat(options)
```

**Reactive state:**

| Property | Type | Description |
|----------|------|-------------|
| `messages` | `Ref<Message[]>` | Messages in the current dialog |
| `participatingDialogs` | `Ref<DialogListItem[]>` | Dialogs user participates in |
| `availableDialogs` | `Ref<DialogListItem[]>` | Dialogs user can join |
| `archivedDialogs` | `Ref<DialogListItem[]>` | Archived dialogs |
| `participants` | `Ref<DialogParticipant[]>` | Current dialog participants |
| `currentDialog` | `Ref<DialogListItem \| null>` | Selected dialog |
| `isConnected` | `Ref<boolean>` | WebSocket connection status |
| `isLoading` | `ComputedRef<boolean>` | Any loading operation in progress |
| `error` | `Ref<Error \| null>` | Last error |
| `firstUnreadMessageId` | `Ref<string \| null>` | First unread message ID |
| `replyToMessage` | `Ref<Message \| null>` | Message being replied to |
| `editingMessage` | `Ref<Message \| null>` | Message being edited |
| `searchQuery` | `Ref<string>` | Current search filter |
| `onlineUsers` | `Ref<Set<string>>` | Set of online user IDs |
| `hasMoreMessages` | `Ref<boolean>` | More older messages available |
| `hasMoreAfter` | `Ref<boolean>` | More newer messages available |

**Methods:**

| Method | Description |
|--------|-------------|
| `connect()` | Open WebSocket connection |
| `disconnect()` | Close WebSocket connection |
| `sendMessage(content, attachments?)` | Send a message |
| `editMessage(messageId, content)` | Edit a message |
| `deleteMessage(messageId)` | Delete a message |
| `selectDialog(dialogId)` | Select and load a dialog |
| `joinDialog(dialogId, profile)` | Join a dialog |
| `leaveDialog(dialogId)` | Leave a dialog |
| `archiveDialog(dialogId)` | Archive a dialog |
| `unarchiveDialog(dialogId)` | Unarchive a dialog |
| `pinDialog(dialogId)` | Pin a dialog |
| `unpinDialog(dialogId)` | Unpin a dialog |
| `toggleNotifications(dialogId)` | Toggle notification mute |
| `markAsRead(messageId?)` | Mark messages as read |
| `loadOlderMessages()` | Load older messages (scroll up) |
| `loadNewerMessages()` | Load newer messages (after jump) |
| `jumpToMessage(messageId)` | Jump to a specific message |
| `resetToLatest()` | Return to latest messages |
| `setReplyTo(message)` | Set reply target |
| `clearReplyTo()` | Clear reply target |
| `setEditMessage(message)` | Enter edit mode |
| `clearEditMessage()` | Exit edit mode |
| `setSearchQuery(query)` | Filter dialogs |
| `isUserOnline(userId)` | Check if user is online |

### `useFileUpload`

File upload management composable:

```typescript
import { useFileUpload, type UseFileUploadOptions } from '@mtchat/vue'

const upload = useFileUpload({
  config: chatConfig,
  dialogId: 'dialog-1',
})

// Add files (from input or drag & drop)
upload.addFiles(fileList)

// Upload all pending files
const results = await upload.uploadAll()

// Send message with uploaded attachments
const attachments = upload.pendingFiles.value
  .filter(f => f.status === 'uploaded')
  .map(f => ({
    s3_key: f.s3Key!,
    filename: f.filename,
    content_type: f.contentType,
    size: f.size,
  }))

await chat.sendMessage('Here are the files', attachments)
upload.clearAll()
```

**State:**

| Property | Type | Description |
|----------|------|-------------|
| `pendingFiles` | `Ref<PendingAttachment[]>` | Files pending upload |
| `isUploading` | `Ref<boolean>` | Upload in progress |

**Methods:**

| Method | Description |
|--------|-------------|
| `addFiles(files)` | Add files to the upload queue |
| `removeFile(id)` | Remove a file from the queue |
| `uploadAll()` | Upload all pending files to S3 |
| `retryUpload(id)` | Retry a failed upload |
| `clearAll()` | Clear all files from the queue |
