# Компоненты

## MTChat

Основной компонент с двумя режимами отображения.

### Полный режим

Боковая панель со списком чатов и область чата:

```vue
<template>
  <div style="height: 600px;">
    <MTChat
      :config="config"
      mode="full"
      theme="light"
      @connected="onConnected"
      @message-sent="onMessageSent"
    />
  </div>
</template>

<script setup lang="ts">
import { MTChat } from '@mtchat/vue'

const config = {
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
  locale: 'ru',
}
</script>
```

`@mtchat/vue` инжектит стили из package bundle. Отдельный CSS-импорт не требуется.

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
| `showSidebar` | `boolean` | `true` | Показать/скрыть боковую панель (только полный режим; в inline-режиме отключается автоматически) |
| `showTabs` | `boolean` | `true` | Полный режим: показать переключатель вкладок (Участвую / Доступные). Влияет только на UI |
| `showSearch` | `boolean` | `true` | Полный режим: показать поле поиска по диалогам. Влияет только на UI |
| `searchPlaceholder` | `string` | -- | Полный режим: переопределить placeholder поля поиска |
| `showContextMenu` | `boolean` | `true` | Полный режим: включить контекстное меню (ПКМ) на элементах списка диалогов |
| `theme` | `string` | `'light'` | Тема: `'light'` или `'dark'`. В `<MTChatPrime>` тип сужен до `'light' \| 'dark' \| undefined` (`undefined` — автоопределение по PrimeVue dark mode) |
| `token` | `string` | -- | JWT-токен авторизации, переопределяет `config.token` |
| `interceptObjectNavigation` | `boolean` | `false` | Заменить обычный переход по `<a href>` на ссылке объекта диалога (`object_url`) на клиентский эмит `object-navigate` — см. раздел «События» |

### События

| Событие | Данные | Описание |
|---------|--------|----------|
| `connected` | -- | WebSocket подключён |
| `disconnected` | -- | WebSocket отключён |
| `message-sent` | `Message` | Пользователь отправил сообщение |
| `dialog-selected` | `DialogListItem` | Пользователь выбрал диалог |
| `dialog-joined` | `string` | Пользователь присоединился к диалогу |
| `dialog-left` | `string` | Пользователь покинул диалог |
| `object-navigate` | `{ dialog: DialogListItem, originalEvent: MouseEvent }` | Клик по ссылке объекта диалога при включённом пропе `interceptObjectNavigation` |

При `interceptObjectNavigation: true` ссылка на объект диалога (в заголовке чата и в панели информации) вместо обычного `<a href target="_blank">` рендерится как `<button>` без `href` и показывается безусловно — независимо от того, задан ли у диалога `object_url`. Клик по ней эмитит `object-navigate` с полным объектом диалога и исходным `MouseEvent`, чтобы хост мог сам выполнить переход (например, через `vue-router`). В inline-режиме проп не действует — там ссылка на объект не показывается в принципе.

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
  participatingDialogs,
  availableDialogs,
  messages,
  currentDialog,
  isConnected,
  sendMessage,
  joinDialog,
  leaveDialog,
  selectDialog,
  loadOlderMessages,
  loadNewerMessages,
  jumpToMessage,
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
