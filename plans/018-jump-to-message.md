# Plan 018: Jump to Unloaded Reply Message

**Status:** Implemented
**Date:** 2026-02-13

## Summary

When clicking on a quoted message that isn't loaded in the DOM (due to pagination), load messages centered around the target and scroll to it.

## Problem

- Chat has 250 messages, only last 50 loaded initially
- Message 250 replies to message 1
- Click on quoted message → nothing happens (message 1 not in DOM)
- User sees "..." or "Message deleted" instead of actual quote content

## Solution

Add `around` parameter to messages API for loading messages centered around a target message ID.

## Key Changes

### 1. Backend: message_repo.rs

New `list_around()` method:

```rust
pub async fn list_around(
    &self,
    dialog_id: Uuid,
    around_id: Uuid,
    limit: i64,
) -> Result<(Vec<Message>, bool, bool), sqlx::Error> {
    let half_limit = limit / 2;

    // Get messages BEFORE the target (not including target)
    let before_messages = sqlx::query_as(
        "SELECT * FROM messages WHERE dialog_id = $1 AND id < $2 ORDER BY id DESC LIMIT $3"
    )...

    // Get messages FROM the target onwards (including target)
    let after_messages = sqlx::query_as(
        "SELECT * FROM messages WHERE dialog_id = $1 AND id >= $2 ORDER BY id ASC LIMIT $3"
    )...

    // Return (messages, has_more_before, has_more_after)
    Ok((combined, has_more_before, has_more_after))
}
```

### 2. Backend: main.rs

Updated `PaginationQuery`:

```rust
struct PaginationQuery {
    limit: i64,
    before: Option<Uuid>,
    around: Option<Uuid>,  // NEW
}

struct MessagesResponse {
    messages: Vec<MessageWithAttachments>,
    first_unread_message_id: Option<Uuid>,
    has_more_before: Option<bool>,  // NEW
    has_more_after: Option<bool>,   // NEW
}
```

Updated `list_messages` handler to support both pagination modes:
- Regular: `before` parameter (infinite scroll)
- Centered: `around` parameter (jump to message)

### 3. SDK: types/index.ts

```typescript
export interface PaginationOptions {
  limit?: number
  before?: string
  around?: string  // NEW
}

export interface MessagesResponse {
  messages: Message[]
  first_unread_message_id?: string
  has_more_before?: boolean  // NEW
  has_more_after?: boolean   // NEW
}
```

### 4. SDK: api.ts

Updated `getMessages()`:

```typescript
async getMessages(dialogId: string, options?: PaginationOptions): Promise<MessagesResponse> {
  const params: Record<string, string> = {}
  if (options?.limit) params.limit = String(options.limit)
  if (options?.before) params.before = options.before
  if (options?.around) params.around = options.around  // NEW
  // ...
}
```

### 5. Composable: useChat.ts

New state and method:

```typescript
const isJumpingToMessage = ref(false)

async function jumpToMessage(messageId: string): Promise<boolean> {
  // 1. Check if already loaded → return true
  if (messages.value.some(m => m.id === messageId)) return true

  // 2. Check if deleted (from cache) → return false
  if (replyMessagesCache.value.get(messageId) === null) return false

  // 3. Load messages around target
  isJumpingToMessage.value = true
  try {
    const response = await client.api.getMessages(currentDialog.value.id, {
      around: messageId,
      limit: 50
    })

    // 4. Replace messages with new set
    messages.value = response.messages
    hasMoreMessages.value = response.has_more_before ?? true

    return response.messages.some(m => m.id === messageId)
  } finally {
    isJumpingToMessage.value = false
  }
}
```

### 6. Component: ChatMessages.vue

New prop and emit:

```typescript
const props = defineProps<{
  // ...existing
  isJumpingToMessage: boolean
}>()

const emit = defineEmits<{
  // ...existing
  jumpToMessage: [messageId: string]
}>()
```

New click handler:

```typescript
function handleQuotedClick(replyToId: string) {
  const replyMsg = getReplyMessage(replyToId)
  if (replyMsg === null) return  // Deleted, do nothing

  const loaded = props.messages.some(m => m.id === replyToId)
  if (loaded) {
    scrollToMessage(replyToId)
  } else {
    emit('jumpToMessage', replyToId)
  }
}
```

Loading overlay:

```vue
<div v-if="isJumpingToMessage" class="chat-messages__loading-overlay">
  <span>{{ t.chat.loadingOlder }}</span>
</div>
```

### 7. Component: MTChat.vue

Handler for jump event:

```typescript
async function handleJumpToMessage(messageId: string) {
  const found = await chat.jumpToMessage(messageId)
  if (found) {
    await nextTick()
    messagesRef.value?.scrollToMessage(messageId)
  }
}
```

Connected to ChatMessages:

```vue
<ChatMessages
  :is-jumping-to-message="chat.isJumpingToMessage.value"
  @jump-to-message="handleJumpToMessage"
/>
```

## Flow

1. User clicks quoted message
2. `handleQuotedClick()` checks if message is loaded
3. If not loaded, emits `jumpToMessage` event
4. Parent calls `chat.jumpToMessage(messageId)`
5. API fetches messages centered around target (`?around=uuid`)
6. Messages replaced in state
7. After `nextTick()`, scroll to target message
8. Message highlighted with animation

## Edge Cases

| Case | Behavior |
|------|----------|
| Message in DOM | Direct scroll, no API call |
| Message deleted | No action (cache returns null) |
| Message loading | Shows "..." in quote |
| API error | Logs warning, returns false |

## Bidirectional Infinite Scroll

After jumping to a message, the user may be in the middle of the conversation. They need to be able to scroll both up (older) and down (newer) to load more messages.

### Backend Changes

Added `after` parameter to `PaginationQuery`:

```rust
struct PaginationQuery {
    limit: i64,
    before: Option<Uuid>,
    after: Option<Uuid>,   // NEW - load newer messages
    around: Option<Uuid>,
}
```

New `list_after()` method in message_repo.rs:

```rust
pub async fn list_after(
    &self,
    dialog_id: Uuid,
    after_id: Uuid,
    limit: i64,
) -> Result<Vec<Message>, sqlx::Error> {
    // SELECT * FROM messages WHERE dialog_id = $1 AND id > $2 ORDER BY id ASC LIMIT $3
}
```

### Frontend Changes

New state in useChat.ts:

```typescript
const hasMoreAfter = ref(false)      // more messages after (newer)
const isLoadingNewer = ref(false)
const newestMessageId = ref<string | null>(null)
```

New methods:

- `loadNewerMessages()` - load messages after newest loaded message
- `resetToLatest()` - reset to latest messages (for scroll-to-bottom button)

### Scroll-to-Bottom Button Behavior

When `hasMoreAfter` is true (we jumped up in the conversation):
- Button calls `resetToLatest()` to reload latest messages
- Then scrolls to bottom

When `hasMoreAfter` is false (already at latest):
- Button just scrolls to bottom of current messages

## Verification

1. Create chat with 100+ messages via demo app
2. Send reply from last message to first message
3. Click quoted message → should load and scroll
4. Check loading indicator appears during load
5. Test clicking deleted message quote → nothing happens
6. Verify infinite scroll works after jump:
   - Scroll UP loads older messages
   - Scroll DOWN loads newer messages
7. Test scroll-to-bottom button after jump → should reset to latest
8. Test with virtual scroll enabled (100+ messages)
