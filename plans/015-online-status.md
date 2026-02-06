# Plan 015: User Online Status

## Overview

Реализовать индикатор онлайн-статуса пользователей:
- Зелёная точка около имени отправителя в сообщениях
- Зелёный индикатор около аватара в списке участников
- Кеширование в Redis с коротким TTL (60s)
- Real-time обновления через WebSocket

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                         Redis                                │
│   Keys: online:{user_id} = "1"  TTL: 60s                    │
│   - SET on WS connect                                        │
│   - EXPIRE refresh on WS ping (30s heartbeat)               │
│   - DEL on WS disconnect                                     │
└─────────────────────────────────────────────────────────────┘
                    ▲                    │
                    │ SET/EXPIRE/DEL     │ presence.update event
                    │ MGET batch         ▼
┌─────────────────────────────────────────────────────────────┐
│                    Backend (Rust)                            │
│  ws.rs: track connect/ping/disconnect                       │
│  presence.rs: Redis operations                              │
│  API: is_online field in participants                       │
└─────────────────────────────────────────────────────────────┘
                    │
                    │ WS: { type: "presence.update", user_id, is_online }
                    ▼
┌─────────────────────────────────────────────────────────────┐
│                    Frontend (Vue)                            │
│  useChat: onlineUsers Set, isUserOnline()                   │
│  ChatInfoPanel: green dot on avatar                         │
│  MTChat: green dot before sender name                       │
└─────────────────────────────────────────────────────────────┘
```

---

## Implementation

### 1. Backend: Presence Service

**File (NEW):** `mtchat-rust/src/services/presence.rs`

```rust
use fred::prelude::*;
use uuid::Uuid;

const ONLINE_TTL: i64 = 60;

pub struct PresenceService {
    redis: RedisPool,
}

impl PresenceService {
    pub fn new(redis: RedisPool) -> Self { Self { redis } }

    pub async fn set_online(&self, user_id: Uuid) -> Result<(), RedisError> {
        let key = format!("online:{}", user_id);
        self.redis.set(&key, "1", Some(Expiration::EX(ONLINE_TTL)), None, false).await
    }

    pub async fn refresh_online(&self, user_id: Uuid) -> Result<(), RedisError> {
        let key = format!("online:{}", user_id);
        self.redis.expire(&key, ONLINE_TTL).await
    }

    pub async fn set_offline(&self, user_id: Uuid) -> Result<(), RedisError> {
        let key = format!("online:{}", user_id);
        self.redis.del(&key).await
    }

    pub async fn get_online_users(&self, user_ids: &[Uuid]) -> Result<Vec<Uuid>, RedisError> {
        if user_ids.is_empty() { return Ok(vec![]); }
        let keys: Vec<String> = user_ids.iter().map(|id| format!("online:{}", id)).collect();
        let results: Vec<Option<String>> = self.redis.mget(&keys).await?;
        Ok(user_ids.iter().zip(results).filter_map(|(id, r)| r.map(|_| *id)).collect())
    }
}
```

**File:** `mtchat-rust/src/services/mod.rs` — добавить `pub mod presence;`

---

### 2. Backend: Redis в AppState

**File:** `mtchat-rust/src/main.rs`

```rust
use fred::prelude::*;
use services::PresenceService;

struct AppState {
    // ... existing
    redis: RedisPool,
    presence: Arc<PresenceService>,
}

// В main():
let redis_config = RedisConfig::from_url(&env::var("REDIS_URL").unwrap_or("redis://localhost:6379".into()))?;
let redis = Builder::from_config(redis_config).build_pool(5)?;
redis.init().await?;
let presence = Arc::new(PresenceService::new(redis.clone()));
```

---

### 3. Backend: WebSocket Online Tracking

**File:** `mtchat-rust/src/ws.rs`

Добавить событие:
```rust
#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsEvent {
    // ... existing
    #[serde(rename = "presence.update")]
    PresenceUpdate { user_id: Uuid, is_online: bool },
}
```

Обновить `handle_socket`:
```rust
pub async fn handle_socket(
    socket: WebSocket,
    connections: Connections,
    user_id: Uuid,
    presence: Arc<PresenceService>,
    participants: Arc<ParticipantRepository>,
) {
    // ... setup ...

    // On connect
    presence.set_online(user_id).await.ok();
    broadcast_presence(&connections, &participants, user_id, true).await;

    // On ping (in message handler)
    WsClientMessage::Ping => {
        presence.refresh_online(user_id).await.ok();
        // ... send pong
    }

    // On disconnect (cleanup)
    presence.set_offline(user_id).await.ok();
    broadcast_presence(&connections, &participants, user_id, false).await;
}
```

Broadcast функция:
```rust
async fn broadcast_presence(
    connections: &Connections,
    participants: &ParticipantRepository,
    user_id: Uuid,
    is_online: bool,
) {
    // Get dialogs user participates in
    let dialog_ids = participants.get_user_dialogs(user_id).await.unwrap_or_default();
    // Get all participants in those dialogs
    let recipient_ids = participants.get_dialog_participants_user_ids(&dialog_ids).await.unwrap_or_default();
    // Broadcast to connected recipients
    let event = WsEvent::PresenceUpdate { user_id, is_online };
    let json = serde_json::to_string(&event).unwrap();
    let conns = connections.read().await;
    for rid in recipient_ids {
        if rid != user_id {
            if let Some(tx) = conns.get(&rid) {
                tx.send(json.clone()).await.ok();
            }
        }
    }
}
```

---

### 4. Backend: Repository Methods

**File:** `mtchat-rust/src/repositories/participant_repo.rs`

```rust
pub async fn get_user_dialogs(&self, user_id: Uuid) -> Result<Vec<Uuid>, sqlx::Error> {
    sqlx::query_scalar("SELECT dialog_id FROM dialog_participants WHERE user_id = $1")
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
}

pub async fn get_dialog_participants_user_ids(&self, dialog_ids: &[Uuid]) -> Result<Vec<Uuid>, sqlx::Error> {
    sqlx::query_scalar("SELECT DISTINCT user_id FROM dialog_participants WHERE dialog_id = ANY($1)")
        .bind(dialog_ids)
        .fetch_all(&self.pool)
        .await
}
```

---

### 5. Backend: API — is_online в participants

**File:** `mtchat-rust/src/main.rs`

```rust
#[derive(Serialize)]
struct ParticipantResponse {
    #[serde(flatten)]
    participant: DialogParticipant,
    is_online: bool,
}

async fn list_participants(...) -> Result<Json<Vec<ParticipantResponse>>, ApiError> {
    let participants = state.participants.list_by_dialog(dialog_id).await?;
    let user_ids: Vec<Uuid> = participants.iter().map(|p| p.user_id).collect();
    let online_users = state.presence.get_online_users(&user_ids).await.unwrap_or_default();

    let responses = participants.into_iter().map(|p| ParticipantResponse {
        is_online: online_users.contains(&p.user_id),
        participant: p,
    }).collect();

    Ok(Json(responses))
}
```

---

### 6. Frontend: Types

**File:** `mtchat-vue/src/types/index.ts`

```typescript
export interface DialogParticipant {
  // ... existing fields
  is_online?: boolean  // NEW
}

export type WsEventType =
  // ... existing
  | 'presence.update'  // NEW
```

---

### 7. Frontend: useChat

**File:** `mtchat-vue/src/composables/useChat.ts`

```typescript
// State
const onlineUsers = ref<Set<string>>(new Set())

// Handler
function handlePresenceUpdate(event: { user_id?: string; is_online?: boolean }) {
  if (!event.user_id) return
  const newSet = new Set(onlineUsers.value)
  if (event.is_online) {
    newSet.add(event.user_id)
  } else {
    newSet.delete(event.user_id)
  }
  onlineUsers.value = newSet

  // Update participant in list
  const idx = participants.value.findIndex(p => p.user_id === event.user_id)
  if (idx !== -1) {
    participants.value[idx] = { ...participants.value[idx], is_online: event.is_online }
  }
}

// Helper
function isUserOnline(userId: string): boolean {
  return onlineUsers.value.has(userId)
}

// Register handler
client.on('presence.update', handlePresenceUpdate)

// In loadParticipants — populate onlineUsers from is_online field
const online = loadedParticipants.filter(p => p.is_online).map(p => p.user_id)
onlineUsers.value = new Set(online)

// Export
return { onlineUsers, isUserOnline, ... }
```

---

### 8. Frontend: ChatInfoPanel (participant list)

**File:** `mtchat-vue/src/components/chat/ChatInfoPanel.vue`

```vue
<div class="chat-info-panel__avatar-wrapper">
  <div class="chat-info-panel__avatar">{{ initials }}</div>
  <span v-if="participant.is_online" class="chat-info-panel__online-indicator"></span>
</div>

<style>
.chat-info-panel__avatar-wrapper { position: relative; }
.chat-info-panel__online-indicator {
  position: absolute;
  bottom: 2px;
  right: 2px;
  width: 10px;
  height: 10px;
  background: #4CAF50;
  border: 2px solid var(--mtchat-bg);
  border-radius: 50%;
}
</style>
```

---

### 9. Frontend: MTChat (message sender)

**File:** `mtchat-vue/src/components/MTChat.vue`

```vue
<span class="mtchat__message-sender">
  <span v-if="message.sender_id && chat.isUserOnline(message.sender_id)"
        class="mtchat__online-dot"></span>
  {{ getSenderDisplayName(message.sender_id) }}
</span>

<style>
.mtchat__online-dot {
  display: inline-block;
  width: 8px;
  height: 8px;
  background: #4CAF50;
  border-radius: 50%;
  margin-right: 4px;
}
</style>
```

---

## Files to Modify

### Backend (mtchat-rust)
| File | Changes |
|------|---------|
| `src/services/mod.rs` | Add `pub mod presence;` |
| `src/services/presence.rs` | NEW: PresenceService with Redis ops |
| `src/main.rs` | Add Redis pool, presence service to AppState; update ws_handler call; update list_participants |
| `src/ws.rs` | Add PresenceUpdate event; track online on connect/ping/disconnect; broadcast_presence fn |
| `src/repositories/participant_repo.rs` | Add get_user_dialogs, get_dialog_participants_user_ids |

### Frontend (mtchat-vue)
| File | Changes |
|------|---------|
| `src/types/index.ts` | Add is_online to DialogParticipant; add presence.update to WsEventType |
| `src/composables/useChat.ts` | Add onlineUsers, handlePresenceUpdate, isUserOnline |
| `src/components/chat/ChatInfoPanel.vue` | Add online indicator to avatar |
| `src/components/MTChat.vue` | Add online dot to message sender |

---

## Verification

1. `docker-compose up -d` (Redis должен быть запущен)
2. `cd mtchat-rust && cargo run`
3. `cd mtchat-example && npm run dev`
4. Тесты:
   - [ ] Зелёный индикатор появляется при подключении через WS
   - [ ] Индикатор пропадает при закрытии вкладки
   - [ ] Индикатор виден в списке участников (ChatInfoPanel)
   - [ ] Зелёная точка видна около имени в сообщениях
   - [ ] Real-time обновление при online/offline другого пользователя
   - [ ] Статус сохраняется при обновлении страницы (Redis TTL)

---

## Performance

- **Redis TTL 60s** — достаточный буфер над heartbeat 30s
- **Batch MGET** — один запрос Redis на весь список участников
- **Targeted broadcast** — только пользователям в общих диалогах
- **Нет DB запросов** для проверки онлайн-статуса
