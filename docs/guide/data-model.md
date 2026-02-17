# Data Model

## Core Entities

### Dialog

A dialog is a conversation bound to a business object in your system.

```
┌─────────────────────────────────────────────┐
│                   Dialog                     │
├─────────────────────────────────────────────┤
│  id              UUID (v7, time-ordered)     │
│  object_id       UUID         (required)     │
│  object_type     STRING       (required)     │
│  title           STRING                      │
│  object_url      STRING       (optional)     │
│  created_by      UUID                        │
│  created_at      TIMESTAMP                   │
└─────────────────────────────────────────────┘
```

- **object_id** + **object_type** link the dialog to an entity in your system (e.g., an order, tender, or route)
- Multiple dialogs can be created for the same object
- **object_url** is an optional link back to the object in the host application

### Participant

A direct member of a dialog. Participants receive notifications and see the dialog in their "My Chats" list.

```
┌─────────────────────────────────────────────┐
│               Participant                    │
├─────────────────────────────────────────────┤
│  dialog_id             UUID                  │
│  user_id               UUID                  │
│  display_name          STRING                │
│  company               STRING                │
│  email                 STRING (optional)      │
│  phone                 STRING (optional)      │
│  joined_at             TIMESTAMP              │
│  joined_as             "creator" | "member"   │
│  notifications_enabled BOOLEAN                │
│  last_read_msg_id      UUID (nullable)        │
│  unread_count          INTEGER                │
│  is_archived           BOOLEAN                │
│  is_pinned             BOOLEAN                │
└─────────────────────────────────────────────┘
```

- **user_id** is an external identifier from your system (not managed by MTChat)
- **display_name** and **company** are set when the user joins
- **unread_count** is tracked per participant
- **is_archived** and **is_pinned** are per-user states

### Access Scope

Defines rules for potential participants -- users who can discover and join the dialog.

```
┌─────────────────────────────────────────────┐
│              Access Scope                    │
├─────────────────────────────────────────────┤
│  dialog_id       UUID                        │
│  tenant_uid      STRING                      │
│  scope_level1    STRING[]    (departments)   │
│  scope_level2    STRING[]    (permissions)   │
└─────────────────────────────────────────────┘
```

A dialog can have multiple access scopes. See [Scope Matching](scope-matching.md) for details on the matching algorithm.

### Message

```
┌─────────────────────────────────────────────┐
│                  Message                     │
├─────────────────────────────────────────────┤
│  id              UUID (v7, time-ordered)     │
│  dialog_id       UUID                        │
│  sender_id       UUID (nullable for system)  │
│  message_type    "user" | "system"           │
│  content         STRING (HTML)               │
│  reply_to_id     UUID (nullable)             │
│  is_edited       BOOLEAN                     │
│  is_deleted      BOOLEAN                     │
│  created_at      TIMESTAMP                   │
│  updated_at      TIMESTAMP                   │
└─────────────────────────────────────────────┘
```

- User messages contain sanitized HTML (allowed tags: `p`, `br`, `strong`, `em`, `u`, `s`, `a`, `ul`, `ol`, `li`, `blockquote`, `code`, `pre`, `span`)
- System messages (joins, leaves, creation) have `message_type = "system"` and `sender_id = NULL`
- System messages store JSON content for i18n rendering on the frontend
- Edited messages retain `is_edited = true`; deleted messages retain `is_deleted = true`

### Attachment

```
┌─────────────────────────────────────────────┐
│                Attachment                    │
├─────────────────────────────────────────────┤
│  id              UUID (v7)                   │
│  message_id      UUID                        │
│  s3_key          STRING                      │
│  filename        STRING                      │
│  content_type    STRING (MIME)                │
│  size            BIGINT (bytes)              │
│  created_at      TIMESTAMP                   │
└─────────────────────────────────────────────┘
```

- Files are stored in S3/MinIO and accessed via presigned URLs
- Max file size: 100 MB
- Max 10 attachments per message

## Entity Relationships

```
Dialog ──┬── Participants (direct members)
         ├── Access Scopes (who can join)
         └── Messages ── Attachments
                      └── Edit History
```

## Database Tables

| Table | Description |
|-------|-------------|
| `dialogs` | Conversations bound to business objects |
| `dialog_participants` | Direct members with per-user state |
| `dialog_access_scopes` | Scope rules for potential participants |
| `messages` | Messages with reply support |
| `attachments` | File attachments linked to messages |
| `message_edit_history` | History of message edits |

All tables use UUIDv7 for primary keys (time-ordered for better index performance). Migrations run automatically on server startup.
