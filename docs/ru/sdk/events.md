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
| `object-navigate` | `{ dialog: DialogListItem, originalEvent: MouseEvent }` | Клик по ссылке объекта диалога при включённом пропе `interceptObjectNavigation` — см. ниже |

```vue
<MTChat
  :config="config"
  @connected="onConnected"
  @message-sent="onMessageSent"
  @dialog-selected="onDialogSelected"
/>
```

### `object-navigate`

По умолчанию ссылка на объект диалога (в заголовке чата и в панели информации — ссылка на `object_url`, например заказ или тендер, к которому привязан чат) — обычный `<a href target="_blank">`, клик по ней уводит пользователя из SPA.

Проп `interceptObjectNavigation: true` передаёт переход на сторону хоста (например, чтобы сделать переход через `vue-router` вместо полной перезагрузки страницы). При включённом пропе:

- Элемент рендерится как `<button>` (без `href`) — и в иконке заголовка чата, и в ссылке панели информации.
- Показывается **безусловно** — в отличие от дефолтной `<a>`, не зависит от того, задан ли у диалога `object_url`.
- Клик эмитит `object-navigate` с полным объектом диалога и исходным `MouseEvent` (можно проверить `ctrlKey`/`metaKey`, если нужно особое поведение для открытия в новой вкладке) вместо навигации браузера.

В inline-режиме (`mode="inline"`) проп не действует — там ссылка на объект не показывается вовсе, так как хост уже находится на странице объекта.

```vue
<script setup lang="ts">
import { MTChat, type ObjectNavigateEvent } from '@mtchat/vue'
import { useRouter } from 'vue-router'

const router = useRouter()

function onObjectNavigate({ dialog }: ObjectNavigateEvent) {
  router.push({ name: 'order-detail', params: { id: dialog.object_id } })
}
</script>

<template>
  <MTChat
    :config="config"
    :intercept-object-navigation="true"
    @object-navigate="onObjectNavigate"
  />
</template>
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
