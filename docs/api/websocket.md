# WebSocket

MTChat uses WebSocket for real-time communication. The Vue SDK manages the connection automatically, but you can also connect directly.

## Connection

```
WS /api/v1/ws?user_id={uuid}
```

The `user_id` query parameter identifies the connected user. Upon connection, the server sends a `connected` event and sets the user's online status.

### Connection Lifecycle

1. Client connects with `user_id`
2. Server registers the connection and sets user as online
3. Server broadcasts `presence.update` to other users in shared dialogs
4. Client sends `ping` messages every 30 seconds to maintain presence
5. On disconnect, server removes the connection and broadcasts offline status

## Server Events

Events sent from the server to connected clients.

### connected

Sent immediately after connection is established.

```json
{
  "type": "connected",
  "employee_id": "11111111-1111-1111-1111-111111111111"
}
```

### message.new

A new message was sent in a dialog.

```json
{
  "type": "message.new",
  "id": "019481b3-...",
  "dialog_id": "019481a2-...",
  "sender_id": "11111111-...",
  "content": "<p>Hello!</p>",
  "sent_at": "2026-02-17T12:10:00Z",
  "message_type": "user"
}
```

For system messages (join/leave notifications), `sender_id` is `null` and `message_type` is `"system"`.

### message.edited

A message was edited.

```json
{
  "type": "message.edited",
  "id": "019481b3-...",
  "dialog_id": "019481a2-...",
  "content": "<p>Updated content</p>",
  "last_edited_at": "2026-02-17T12:15:00Z"
}
```

### message.deleted

A message was deleted.

```json
{
  "type": "message.deleted",
  "id": "019481b3-...",
  "dialog_id": "019481a2-..."
}
```

### message.read

A user's read position was updated (read receipt).

```json
{
  "type": "message.read",
  "dialog_id": "019481a2-...",
  "user_id": "11111111-...",
  "last_read_message_id": "019481b3-..."
}
```

### participant.joined

A user joined a dialog.

```json
{
  "type": "participant.joined",
  "dialog_id": "019481a2-...",
  "user_id": "11111111-..."
}
```

### participant.left

A user left a dialog.

```json
{
  "type": "participant.left",
  "dialog_id": "019481a2-...",
  "user_id": "11111111-..."
}
```

### dialog.archived

A dialog was archived (via auto-archive or manual action).

```json
{
  "type": "dialog.archived",
  "dialog_id": "019481a2-..."
}
```

### dialog.unarchived

A dialog was unarchived (e.g., when a new message is sent to an archived dialog).

```json
{
  "type": "dialog.unarchived",
  "dialog_id": "019481a2-..."
}
```

### presence.update

A user's online status changed.

```json
{
  "type": "presence.update",
  "user_id": "11111111-...",
  "is_online": true
}
```

Presence updates are only sent to users who share at least one dialog with the user whose status changed.

### pong

Response to a client `ping` message.

```json
{
  "type": "pong"
}
```

### error

Server-side error.

```json
{
  "type": "error",
  "message": "Invalid message format"
}
```

## Client Messages

Messages sent from the client to the server.

### ping

Heartbeat message. Should be sent every 30 seconds to maintain online status (60-second TTL).

```json
{
  "type": "ping"
}
```

The server responds with a `pong` message and refreshes the user's online status TTL.

### subscribe

Subscribe to events for a specific dialog.

```json
{
  "type": "subscribe",
  "dialog_id": "019481a2-..."
}
```

### unsubscribe

Unsubscribe from events for a specific dialog.

```json
{
  "type": "unsubscribe",
  "dialog_id": "019481a2-..."
}
```

!!! note
    Currently, all connected users receive all events regardless of subscriptions. Subscription-based filtering is planned for a future release.

## Vue SDK Usage

The Vue SDK manages the WebSocket connection automatically:

```typescript
const config: MTChatConfig = {
  baseUrl: 'http://localhost:8080',
  userId: 'user-uuid',
  // ...
  reconnect: true,              // auto-reconnect (default: true)
  reconnectInterval: 3000,      // reconnect delay in ms (default: 3000)
  heartbeatInterval: 30000,     // ping interval in ms (default: 30000)
}
```

To listen for events in your application:

```typescript
const config: MTChatConfig = {
  // ...
  onConnect: () => console.log('Connected'),
  onDisconnect: () => console.log('Disconnected'),
  onMessage: (event) => console.log('WS event:', event),
}
```
