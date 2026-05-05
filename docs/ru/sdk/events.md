# События, Слоты и Composables

## События компонента

| Событие | Данные | Описание |
|---------|--------|----------|
| `connected` | -- | WebSocket подключён |
| `disconnected` | -- | WebSocket отключён |
| `error` | `Error` | Произошла ошибка |
| `message-sent` | `Message` | Сообщение отправлено |
| `dialog-selected` | `DialogListItem` | Диалог выбран |
| `dialog-joined` | `string` | Пользователь присоединился |
| `dialog-left` | `string` | Пользователь вышел |

```vue
<MTChat
  :config="config"
  @connected="onConnected"
  @message-sent="onMessageSent"
  @dialog-selected="onDialogSelected"
/>
```

## Слоты

### `sidebar-action`

```vue
<MTChat :config="config">
  <template #sidebar-action>
    <button @click="createChat">+ Новый чат</button>
  </template>
</MTChat>
```

### `header-menu-actions`

```vue
<MTChat :config="config">
  <template #header-menu-actions="{ dialog, closeMenu, menuItemClass }">
    <button :class="menuItemClass" @click="action(dialog); closeMenu()">
      Действие
    </button>
  </template>
</MTChat>
```

## Composable: useChat

Для кастомного UI без компонента MTChat:

```typescript
import { useChat } from '@mtchat/vue'

const {
  // Реактивное состояние
  participatingDialogs, // Ref<DialogListItem[]>
  availableDialogs,     // Ref<DialogListItem[]>
  archivedDialogs,      // Ref<DialogListItem[]>
  messages,             // Ref<Message[]>
  currentDialog,        // Ref<DialogListItem | null>
  participants,         // Ref<DialogParticipant[]>
  isConnected,          // Ref<boolean>
  isLoading,            // Ref<boolean>
  error,                // Ref<Error | null>
  firstUnreadMessageId, // Ref<string | null>
  onlineUsers,          // Ref<Set<string>>
  hasMoreMessages,      // Ref<boolean>
  hasMoreAfter,         // Ref<boolean>

  // Действия
  sendMessage,          // (content, attachments?) => Promise
  editMessage,          // (messageId, content) => Promise
  deleteMessage,        // (messageId) => Promise
  joinDialog,           // (dialogId, profile) => Promise
  leaveDialog,          // (dialogId) => Promise
  selectDialog,         // (dialogId) => Promise
  markAsRead,           // (messageId?) => Promise
  archiveDialog,        // (dialogId) => Promise
  unarchiveDialog,      // (dialogId) => Promise
  pinDialog,            // (dialogId) => Promise
  unpinDialog,          // (dialogId) => Promise
  toggleNotifications,  // (dialogId) => Promise
  loadOlderMessages,    // () => Promise
  loadNewerMessages,    // () => Promise
  resetToLatest,        // () => Promise
  jumpToMessage,        // (messageId) => Promise
  setSearchQuery,       // (query) => void

  // Утилиты
  isUserOnline,         // (userId) => boolean
  getReplyMessage,      // (messageId) => Message | null
} = useChat({ config, objectId?, objectType? })
```

## Composable: useFileUpload

```typescript
import { useFileUpload } from '@mtchat/vue'

const {
  pendingFiles,    // Ref<PendingFile[]>
  isUploading,     // Ref<boolean>
  addFiles,        // (files: FileList) => void
  removeFile,      // (id: string) => void
  retryFile,       // (id: string) => void
  uploadAll,       // () => Promise<UploadResult[]>
  clearAll,        // () => void
} = useFileUpload({ config, dialogId })
```
