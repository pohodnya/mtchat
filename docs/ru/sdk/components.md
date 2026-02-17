# Компоненты

## MTChat

Основной компонент с двумя режимами отображения.

### Полный режим

Боковая панель со списком чатов и область чата:

```vue
<template>
  <MTChat
    :config="config"
    mode="full"
    theme="light"
    @connected="onConnected"
    @message-sent="onMessageSent"
  />
</template>

<script setup lang="ts">
import { MTChat } from '@mtchat/vue'
import '@mtchat/vue/style.css'

const config = {
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
  locale: 'ru',
}
</script>
```

### Inline-режим

Один чат, привязанный к бизнес-объекту:

```vue
<MTChat
  :config="config"
  mode="inline"
  :object-id="order.id"
  object-type="order"
  theme="light"
/>
```

### Props

| Prop | Тип | По умолчанию | Описание |
|------|-----|--------------|----------|
| `config` | `MTChatConfig` | обязателен | Конфигурация SDK |
| `mode` | `'full' \| 'inline'` | `'full'` | Режим отображения |
| `objectId` | `string` | -- | ID объекта (для inline-режима) |
| `objectType` | `string` | -- | Тип объекта (для inline-режима) |
| `dialogId` | `string` | -- | Начальный ID диалога (полный режим) |
| `showHeader` | `boolean` | `true` | Показать/скрыть заголовок чата |
| `showSidebar` | `boolean` | `true` | Показать/скрыть боковую панель |
| `theme` | `string` | `'light'` | Тема: `'light'` или `'dark'` |

### События

| Событие | Данные | Описание |
|---------|--------|----------|
| `connected` | -- | WebSocket подключён |
| `disconnected` | -- | WebSocket отключён |
| `message-sent` | `Message` | Пользователь отправил сообщение |
| `dialog-selected` | `DialogListItem` | Пользователь выбрал диалог |
| `dialog-joined` | `string` | Пользователь присоединился к диалогу |
| `dialog-left` | `string` | Пользователь покинул диалог |

---

## Возможности чата

### Форматирование

Редактор на базе Tiptap: жирный (Cmd+B), курсив (Cmd+I), подчёркивание (Cmd+U), зачёркивание, ссылки (Cmd+K), списки, цитаты, код, @упоминания.

### Файлы

Drag-and-drop или клик для прикрепления. Изображения, PDF, документы, архивы, аудио, видео. Макс. 100 МБ, 10 файлов на сообщение.

### Действия с сообщениями

- **Ответить** -- иконка ответа на сообщении
- **Редактировать** -- свои сообщения, через меню или Arrow Up
- **Удалить** -- свои сообщения, через меню

### Клавиатурные сочетания

| Сочетание | Действие |
|-----------|----------|
| Enter | Отправить сообщение |
| Shift+Enter | Новая строка |
| Arrow Up | Редактировать последнее сообщение |
| Esc | Отмена редактирования/ответа, закрытие панелей |
| Cmd+K / Ctrl+K | Фокус на поиск |

---

## Headless-использование (Composables)

Для кастомного UI используйте composables напрямую:

### useChat

```typescript
import { useChat } from '@mtchat/vue'

const {
  dialogs, messages, currentDialog, isConnected,
  sendMessage, joinDialog, leaveDialog, selectDialog,
  loadOlderMessages, jumpToMessage,
  destroy,
} = useChat({ config, objectId: 'order-123', objectType: 'order' })
```

### useFileUpload

```typescript
import { useFileUpload } from '@mtchat/vue'

const {
  pendingFiles, isUploading,
  addFiles, removeFile, uploadAll, clearAll,
} = useFileUpload({ config, dialogId })
```
