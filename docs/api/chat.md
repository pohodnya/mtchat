# Chat API

The Chat API is used by the Vue SDK (or any frontend client) to interact with dialogs and messages in the context of an authenticated user.

## User Identity

The current user is identified by the `user_id` query parameter passed with each request. The scope configuration is passed via the `X-Scope-Config` header (base64-encoded JSON).

The Vue SDK handles this automatically based on the `config.userId` and `config.scopeConfig` values.

---

## List Dialogs

Returns dialogs the user participates in or can join.

```
GET /api/v1/dialogs?type=participating&user_id={uuid}
GET /api/v1/dialogs?type=available&user_id={uuid}
```

### Query Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `type` | string | `participating` | `participating` (my chats) or `available` (can join) |
| `user_id` | UUID | required | Current user's ID |
| `search` | string | -- | Search by dialog title or participant company |
| `archived` | boolean | -- | Filter archived dialogs (`true` for archived only) |

### Response

```json
{
  "data": [
    {
      "id": "019481a2-...",
      "object_id": "550e8400-...",
      "object_type": "order",
      "title": "Order #1234 Discussion",
      "created_at": "2026-02-17T12:00:00Z",
      "participants_count": 3,
      "i_am_participant": true,
      "unread_count": 5,
      "is_archived": false,
      "is_pinned": true,
      "notifications_enabled": true,
      "last_message_at": "2026-02-17T14:30:00Z"
    }
  ]
}
```

| Field | Type | Description |
|-------|------|-------------|
| `participants_count` | integer | Total number of participants |
| `i_am_participant` | boolean | Whether the current user is a participant |
| `can_join` | boolean | Whether the current user can join (available dialogs only) |
| `unread_count` | integer | Unread message count for this user |
| `is_archived` | boolean | Whether this user archived the dialog |
| `is_pinned` | boolean | Whether this user pinned the dialog |
| `notifications_enabled` | boolean | Whether notifications are enabled for this user |
| `last_message_at` | datetime | Timestamp of the last message |

---

## Get Dialog

```
GET /api/v1/dialogs/{id}?user_id={uuid}
```

Returns a single dialog with the same fields as the list response.

---

## Get Dialog by Object

Returns the most recent dialog for a given business object. Used by the SDK in inline mode.

```
GET /api/v1/dialogs/by-object/{object_type}/{object_id}?user_id={uuid}
```

---

## Join Dialog

Joins the current user to a dialog as a participant.

```
POST /api/v1/dialogs/{id}/join?user_id={uuid}
```

### Request Body

```json
{
  "display_name": "John Doe",
  "company": "Acme Inc",
  "email": "john@acme.com",
  "phone": "+1234567890"
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `display_name` | string | Yes | Name shown in chat |
| `company` | string | Yes | Company name |
| `email` | string | No | Contact email (visible to other participants) |
| `phone` | string | No | Contact phone (visible to other participants) |

### Response

```json
{
  "data": {
    "user_id": "...",
    "display_name": "John Doe",
    "company": "Acme Inc",
    "joined_as": "member",
    "joined_at": "2026-02-17T12:05:00Z"
  }
}
```

A system message ("John Doe joined the chat") is created automatically. A `participant.joined` WebSocket event and webhook are sent.

---

## Leave Dialog

```
POST /api/v1/dialogs/{id}/leave?user_id={uuid}
```

Removes the current user from the dialog. A system message and `participant.left` events are sent.

---

## Mark Messages as Read

Updates the user's read position in the dialog.

```
POST /api/v1/dialogs/{id}/read?user_id={uuid}
```

### Request Body

```json
{
  "last_read_message_id": "019481b3-..."
}
```

Resets the `unread_count` to 0 and broadcasts a `message.read` WebSocket event to all connected users.

---

## Archive / Unarchive

Archive or unarchive a dialog for the current user. Per-user state -- other participants are not affected.

```
POST /api/v1/dialogs/{id}/archive?user_id={uuid}
POST /api/v1/dialogs/{id}/unarchive?user_id={uuid}
```

---

## Pin / Unpin

Pin or unpin a dialog for the current user. Pinned dialogs appear at the top of the chat list.

```
POST /api/v1/dialogs/{id}/pin?user_id={uuid}
POST /api/v1/dialogs/{id}/unpin?user_id={uuid}
```

---

## Set Notification Preference

Enable or disable notifications for a specific dialog.

```
POST /api/v1/dialogs/{id}/notifications?user_id={uuid}
```

### Request Body

```json
{
  "enabled": false
}
```

---

## List Participants

Returns all participants of a dialog. Requires the user to be a participant.

```
GET /api/v1/dialogs/{id}/participants?user_id={uuid}
```

### Response

```json
{
  "data": [
    {
      "user_id": "11111111-...",
      "display_name": "Alice",
      "company": "Acme Inc",
      "email": "alice@acme.com",
      "joined_as": "creator",
      "joined_at": "2026-02-17T12:00:00Z",
      "is_online": true
    }
  ]
}
```

---

## List Messages

Returns messages in a dialog with pagination. Requires the user to be a participant.

```
GET /api/v1/dialogs/{dialog_id}/messages?user_id={uuid}
```

### Query Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `limit` | integer | 50 | Number of messages to return |
| `before` | UUID | -- | Load messages before this message ID (scroll up) |
| `after` | UUID | -- | Load messages after this message ID (scroll down) |
| `around` | UUID | -- | Load messages centered around this message ID (jump to message) |

### Response

```json
{
  "data": {
    "messages": [
      {
        "id": "019481b3-...",
        "dialog_id": "019481a2-...",
        "sender_id": "11111111-...",
        "message_type": "user",
        "content": "<p>Hello!</p>",
        "reply_to_id": null,
        "is_edited": false,
        "is_deleted": false,
        "sent_at": "2026-02-17T12:10:00Z",
        "attachments": [
          {
            "id": "019481c4-...",
            "filename": "report.pdf",
            "content_type": "application/pdf",
            "size": 245760,
            "url": "https://s3.example.com/...",
            "thumbnail_url": null
          }
        ]
      }
    ],
    "first_unread_message_id": "019481b5-...",
    "has_more_before": true,
    "has_more_after": false
  }
}
```

| Field | Type | Description |
|-------|------|-------------|
| `first_unread_message_id` | UUID | First unread message for this user (initial load only) |
| `has_more_before` | boolean | Whether older messages are available |
| `has_more_after` | boolean | Whether newer messages are available |

---

## Send Message

Sends a message in a dialog. Requires the user to be a participant.

```
POST /api/v1/dialogs/{dialog_id}/messages?user_id={uuid}
```

### Request Body

```json
{
  "content": "<p>Hello, this is a <strong>formatted</strong> message.</p>",
  "reply_to": "019481b3-...",
  "attachments": [
    {
      "s3_key": "dialogs/019481a2-.../pending/019481d5-....pdf",
      "filename": "report.pdf",
      "content_type": "application/pdf",
      "size": 245760
    }
  ]
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `content` | string | Yes | Message content (HTML, sanitized server-side) |
| `reply_to` | UUID | No | ID of the message being replied to |
| `attachments` | array | No | Files previously uploaded via presigned URL |

Content is sanitized on the server. Allowed HTML tags: `p`, `br`, `strong`, `em`, `u`, `s`, `a`, `ul`, `ol`, `li`, `blockquote`, `code`, `pre`, `span`.

---

## Get Message

```
GET /api/v1/dialogs/{dialog_id}/messages/{id}?user_id={uuid}
```

---

## Edit Message

Edits an existing message. Only the message author can edit. System messages cannot be edited.

```
PUT /api/v1/dialogs/{dialog_id}/messages/{id}?user_id={uuid}
```

### Request Body

```json
{
  "content": "<p>Updated message content.</p>"
}
```

Sets `is_edited = true` and broadcasts a `message.edited` WebSocket event. The original content is saved in the edit history table.

---

## Delete Message

Soft-deletes a message. Only the message author can delete. System messages cannot be deleted.

```
DELETE /api/v1/dialogs/{dialog_id}/messages/{id}?user_id={uuid}
```

Sets `is_deleted = true` and broadcasts a `message.deleted` WebSocket event.

---

## Error Responses

```json
{
  "error": {
    "code": "FORBIDDEN",
    "message": "Not a participant. Join the dialog first."
  }
}
```

| HTTP Status | Code | Description |
|-------------|------|-------------|
| 400 | `BAD_REQUEST` | Invalid request |
| 403 | `FORBIDDEN` | User is not a participant (message endpoints) |
| 404 | `NOT_FOUND` | Dialog or message not found |
| 500 | `INTERNAL_ERROR` | Server error |
