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
| `config` | `MTChatConfig` | -- | Конфигурация SDK. Обязателен, если не передан `client` |
| `client` | `MTChatClient \| null` | -- | Внешний клиент, которым управляет приложение; `userId`, `userProfile` и `locale` берутся из `client.config`. См. «Общий клиент» ниже |
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
| `token` | `string` | -- | JWT-токен авторизации, переопределяет `config.token`. Игнорируется при переданном `client` |
| `interceptObjectNavigation` | `boolean` | `false` | Заменить обычный переход по `<a href>` на ссылке объекта диалога (`object_url`) на клиентский эмит `object-navigate` — см. раздел «События» |

### Общий клиент

По умолчанию каждый `<MTChat>` сам создаёт, подключает и закрывает свой `MTChatClient`. Если в приложении одновременно открыто несколько чатов (раздел сообщений, чат объекта, бейдж непрочитанных в шапке), можно создать один клиент на вкладку браузера и передавать его всем компонентам пропом `client`:

```vue
<script setup lang="ts">
import { onUnmounted, shallowRef } from 'vue'
import { MTChat, MTChatClient } from '@mtchat/vue'

const client = shallowRef(new MTChatClient(config))
client.value.connect()
onUnmounted(() => client.value.disconnect())
</script>

<template>
  <MTChat :client="client" mode="full" />
</template>
```

С `client`:

- `config` не нужен: `userId`, `userProfile` и `locale` берутся из `client.config`. Если переданы оба пропа, используется `client`.
- Клиентом управляет приложение: компонент его не подключает, не отключает и не пересоздаёт, проп `token` игнорируется. Чтобы сменить токен или скоуп, создайте новый клиент и передайте его -- компонент перевесит обработчики на новый клиент и перезагрузит данные. Открытый диалог остаётся открытым, а если новый клиент получает по нему 403 или 404 (например, другая организация) — закрывается.
- При размонтировании компонент снимает свои обработчики WebSocket с клиента, остальные чаты на том же клиенте продолжают работать.
- Если клиент уже подключён к моменту монтирования, данные загружаются сразу.
- Режим выбирается при создании компонента: переключать смонтированный компонент между `config` и `client` нельзя.
- Без `config` компонент ждёт клиента: пока `client` равен `null` или не передан, показывается пустое состояние и ничего не загружается. Если на смонтированном компоненте `client` становится `null` (например, чаты выключены), диалоги, сообщения и открытый диалог сбрасываются; клиент, переданный позже, загружает данные заново.
- Ответы, которые ещё не пришли от прежнего клиента, после подмены `client` или его сброса в `null` игнорируются: в списках не появятся диалоги прежнего скоупа.
- Бейдж непрочитанных может использовать тот же клиент: `client.api.getUnreadCount()` возвращает число непрочитанных в скоупе клиента (`GET /api/v1/unread`).

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
