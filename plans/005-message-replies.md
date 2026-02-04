# 005: Ответы на сообщения (Message Replies)

## Цели

1. При наведении на сообщение — иконка "Ответить"
2. При клике — сообщение появляется над полем ввода (цитата)
3. Отмена ответа: крестик или Esc
4. Отправленное сообщение содержит цитату (reply_to_id)
5. Цитата в чате truncated до 1 строки, клик скролит к оригиналу
6. Кнопка "Scroll to bottom" при скролле вверх

---

## Текущее состояние

### Уже реализовано

| Компонент | Статус | Файл |
|-----------|--------|------|
| `messages.reply_to_id` в БД | ✅ | migrations/20250203000004 |
| `Message.reply_to_id` в Rust | ✅ | domain/message.rs |
| `sendMessage` с `replyTo` | ✅ | sdk/api.ts |
| `Message.reply_to_id` в TypeScript | ✅ | types/index.ts |

### Реализовано (2025-02-04)

| Компонент | Статус |
|-----------|--------|
| UI: иконка ответа на hover | ✅ |
| UI: цитата над полем ввода | ✅ |
| UI: отмена ответа (крестик, Esc) | ✅ |
| UI: отображение цитаты в сообщении | ✅ |
| UI: клик по цитате → скролл к оригиналу | ✅ |
| UI: кнопка "Scroll to bottom" | ✅ |

---

## Архитектура

### Data Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                      Reply Flow                                  │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  1. Hover message → Show reply icon                             │
│       │                                                          │
│       ▼                                                          │
│  2. Click reply icon                                            │
│       │                                                          │
│       ▼                                                          │
│  3. Set replyToMessage state                                    │
│       │                                                          │
│       ▼                                                          │
│  4. Show quote preview above input                              │
│       │                                                          │
│       ├── Cancel (X or Esc) → Clear replyToMessage              │
│       │                                                          │
│       ▼                                                          │
│  5. Send message with reply_to = message.id                     │
│       │                                                          │
│       ▼                                                          │
│  6. Message displayed with quoted reference                     │
│       │                                                          │
│       ▼                                                          │
│  7. Click quote → Scroll to original message                    │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### State

```typescript
// В useChat composable или MTChat component
const replyToMessage = ref<Message | null>(null)

// Методы
function setReplyTo(message: Message) {
  replyToMessage.value = message
}

function clearReplyTo() {
  replyToMessage.value = null
}

// При отправке
async function sendMessage(content: string) {
  await api.sendMessage(dialogId, content, {
    replyTo: replyToMessage.value?.id,
    attachments: ...
  })
  clearReplyTo()
}
```

---

## Этап 1: Composable Updates

**Файл:** `mtchat-vue/src/composables/useChat.ts`

```typescript
// Новое состояние
const replyToMessage: Ref<Message | null> = ref(null)

// Методы
function setReplyTo(message: Message): void {
  replyToMessage.value = message
}

function clearReplyTo(): void {
  replyToMessage.value = null
}

// Обновить sendMessage
async function sendMessage(content: string, attachments?: AttachmentInput[]): Promise<Message | undefined> {
  // ...existing code...
  const message = await client.api.sendMessage(
    currentDialog.value.id,
    content,
    {
      replyTo: replyToMessage.value?.id,
      attachments
    }
  )
  clearReplyTo() // Сбросить после отправки
  // ...
}

// Добавить в return
return {
  // ...existing
  replyToMessage,
  setReplyTo,
  clearReplyTo,
}
```

---

## Этап 2: Иконка ответа на hover

**Файл:** `mtchat-vue/src/components/MTChat.vue`

### Template

```vue
<div
  v-for="message in chat.messages.value"
  :key="message.id"
  :class="['mtchat__message', { 'mtchat__message--own': message.sender_id === config.userId }]"
>
  <!-- Message actions (visible on hover) -->
  <div class="mtchat__message-actions">
    <button
      class="mtchat__action-btn"
      title="Ответить"
      @click="chat.setReplyTo(message)"
    >
      <svg><!-- reply icon --></svg>
    </button>
  </div>

  <!-- Existing message content -->
  <div class="mtchat__message-sender">...</div>
  <div class="mtchat__message-content">...</div>
</div>
```

### Styles

```css
.mtchat__message {
  position: relative;
}

.mtchat__message-actions {
  position: absolute;
  top: 4px;
  right: 4px;
  opacity: 0;
  transition: opacity 0.2s;
}

.mtchat__message:hover .mtchat__message-actions {
  opacity: 1;
}

.mtchat__action-btn {
  width: 28px;
  height: 28px;
  border: none;
  border-radius: 4px;
  background: var(--mtchat-bg-secondary);
  color: var(--mtchat-text-secondary);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
}

.mtchat__action-btn:hover {
  background: var(--mtchat-primary);
  color: white;
}
```

---

## Этап 3: Цитата над полем ввода

### Template

```vue
<!-- Input Area -->
<div v-if="hasDialog" class="mtchat__input-area">
  <!-- Reply Preview -->
  <div v-if="chat.replyToMessage.value" class="mtchat__reply-preview">
    <div class="mtchat__reply-indicator"></div>
    <div class="mtchat__reply-content">
      <div class="mtchat__reply-author">
        {{ chat.replyToMessage.value.sender_id.slice(0, 8) }}
      </div>
      <div class="mtchat__reply-text">
        {{ truncateText(chat.replyToMessage.value.content, 100) }}
      </div>
    </div>
    <button class="mtchat__reply-cancel" @click="chat.clearReplyTo()">
      <svg><!-- X icon --></svg>
    </button>
  </div>

  <!-- Existing input form -->
  <form class="mtchat__input-form">...</form>
</div>
```

### Styles

```css
.mtchat__reply-preview {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  background: var(--mtchat-bg-secondary);
  border-radius: 8px;
  margin-bottom: 8px;
}

.mtchat__reply-indicator {
  width: 3px;
  height: 100%;
  min-height: 32px;
  background: var(--mtchat-primary);
  border-radius: 2px;
}

.mtchat__reply-content {
  flex: 1;
  min-width: 0;
}

.mtchat__reply-author {
  font-size: 12px;
  font-weight: 600;
  color: var(--mtchat-primary);
}

.mtchat__reply-text {
  font-size: 13px;
  color: var(--mtchat-text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.mtchat__reply-cancel {
  width: 24px;
  height: 24px;
  border: none;
  background: transparent;
  color: var(--mtchat-text-secondary);
  cursor: pointer;
  border-radius: 4px;
}

.mtchat__reply-cancel:hover {
  background: var(--mtchat-bg);
  color: var(--mtchat-text);
}
```

---

## Этап 4: Отмена по Esc

### Script

```typescript
// В setup или onMounted
function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape' && chat.replyToMessage.value) {
    chat.clearReplyTo()
  }
}

onMounted(() => {
  document.addEventListener('keydown', handleKeydown)
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeydown)
})
```

---

## Этап 5: Отображение цитаты в сообщении

### Template

```vue
<div :class="['mtchat__message', ...]">
  <!-- Quoted message (if reply) -->
  <div
    v-if="message.reply_to_id"
    class="mtchat__quoted-message"
    @click="scrollToMessage(message.reply_to_id)"
  >
    <div class="mtchat__quoted-indicator"></div>
    <div class="mtchat__quoted-content">
      <div class="mtchat__quoted-author">
        {{ getMessageAuthor(message.reply_to_id) }}
      </div>
      <div class="mtchat__quoted-text">
        {{ getQuotedText(message.reply_to_id) }}
      </div>
    </div>
  </div>

  <!-- Original message content -->
  <div class="mtchat__message-content">{{ message.content }}</div>
</div>
```

### Helper Functions

```typescript
// Получить текст цитируемого сообщения
function getQuotedText(messageId: string): string {
  const msg = chat.messages.value.find(m => m.id === messageId)
  if (!msg) return 'Сообщение удалено'
  return truncateText(msg.content, 60)
}

function getMessageAuthor(messageId: string): string {
  const msg = chat.messages.value.find(m => m.id === messageId)
  if (!msg) return '...'
  return msg.sender_id === config.userId ? 'Вы' : msg.sender_id.slice(0, 8)
}

function truncateText(text: string, maxLength: number): string {
  if (text.length <= maxLength) return text
  return text.slice(0, maxLength) + '...'
}
```

### Styles

```css
.mtchat__quoted-message {
  display: flex;
  gap: 8px;
  padding: 6px 10px;
  margin-bottom: 6px;
  background: rgba(0, 0, 0, 0.1);
  border-radius: 6px;
  cursor: pointer;
  transition: background 0.2s;
}

.mtchat__quoted-message:hover {
  background: rgba(0, 0, 0, 0.15);
}

.mtchat__quoted-indicator {
  width: 2px;
  background: var(--mtchat-primary);
  border-radius: 1px;
}

.mtchat__quoted-content {
  flex: 1;
  min-width: 0;
}

.mtchat__quoted-author {
  font-size: 11px;
  font-weight: 600;
  color: var(--mtchat-primary);
}

.mtchat__quoted-text {
  font-size: 12px;
  color: var(--mtchat-text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
```

---

## Этап 6: Скролл к цитируемому сообщению

### Script

```typescript
const messagesContainer = ref<HTMLElement | null>(null)

function scrollToMessage(messageId: string) {
  const messageEl = messagesContainer.value?.querySelector(
    `[data-message-id="${messageId}"]`
  )
  if (messageEl) {
    messageEl.scrollIntoView({ behavior: 'smooth', block: 'center' })
    // Highlight effect
    messageEl.classList.add('mtchat__message--highlight')
    setTimeout(() => {
      messageEl.classList.remove('mtchat__message--highlight')
    }, 2000)
  }
}
```

### Template Update

```vue
<div
  v-for="message in chat.messages.value"
  :key="message.id"
  :data-message-id="message.id"
  :class="['mtchat__message', ...]"
>
```

### Styles

```css
.mtchat__message--highlight {
  animation: highlight-pulse 2s ease-out;
}

@keyframes highlight-pulse {
  0% { background: rgba(0, 122, 255, 0.3); }
  100% { background: transparent; }
}
```

---

## Этап 7: Кнопка "Scroll to Bottom"

### Template

```vue
<div v-else ref="messagesContainer" class="mtchat__messages" @scroll="handleScroll">
  <!-- Messages... -->

  <!-- Scroll to bottom button -->
  <button
    v-if="showScrollButton"
    class="mtchat__scroll-bottom"
    @click="scrollToBottom"
  >
    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <path d="M12 5v14M5 12l7 7 7-7"/>
    </svg>
  </button>
</div>
```

### Script

```typescript
const showScrollButton = ref(false)

function handleScroll() {
  if (!messagesContainer.value) return

  const { scrollTop, scrollHeight, clientHeight } = messagesContainer.value
  const distanceFromBottom = scrollHeight - scrollTop - clientHeight

  // Show button if scrolled more than 200px from bottom
  showScrollButton.value = distanceFromBottom > 200

  // Existing read tracking logic...
}

function scrollToBottom() {
  if (messagesContainer.value) {
    messagesContainer.value.scrollTo({
      top: messagesContainer.value.scrollHeight,
      behavior: 'smooth'
    })
  }
}
```

### Styles

```css
.mtchat__messages {
  position: relative; /* для позиционирования кнопки */
}

.mtchat__scroll-bottom {
  position: sticky;
  bottom: 16px;
  left: 50%;
  transform: translateX(-50%);
  width: 40px;
  height: 40px;
  border: none;
  border-radius: 50%;
  background: var(--mtchat-primary);
  color: white;
  cursor: pointer;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
  display: flex;
  align-items: center;
  justify-content: center;
  transition: transform 0.2s, box-shadow 0.2s;
  z-index: 10;
}

.mtchat__scroll-bottom:hover {
  transform: translateX(-50%) scale(1.1);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
}
```

---

## Порядок реализации

| # | Задача | Приоритет |
|---|--------|-----------|
| 1 | Composable: replyToMessage state | High |
| 2 | UI: иконка ответа на hover | High |
| 3 | UI: цитата над полем ввода | High |
| 4 | UI: отмена по крестику и Esc | High |
| 5 | Отправка с reply_to | High |
| 6 | UI: цитата в отправленном сообщении | High |
| 7 | Скролл к цитируемому сообщению | Medium |
| 8 | Highlight при скролле | Low |
| 9 | Кнопка "Scroll to bottom" | Medium |
| 10 | Скрытие кнопки при скролле вниз | Medium |

---

## Дополнительные улучшения (Future)

- Swipe right для ответа (mobile)
- Превью изображений в цитате
- Цепочка ответов (thread view)
- Поиск по цитатам
