# Data Model

## Core Entities

### Dialog

A dialog is a conversation bound to a business object in your system.

| Field | Type | Description |
|-------|------|-------------|
| id | UUID (v7) | Time-ordered unique identifier |
| object_id | UUID | Reference to business object (required) |
| object_type | STRING | Type of business object (required) |
| title | STRING | Dialog title |
| object_url | STRING | Link to object in host app (optional) |
| created_by | UUID | User who created the dialog |
| created_at | TIMESTAMP | Creation timestamp |

- **object_id** + **object_type** link the dialog to an entity in your system (e.g., an order, tender, or route)
- Multiple dialogs can be created for the same object
- **object_url** is an optional link back to the object in the host application

### Participant

A direct member of a dialog. Participants receive notifications and see the dialog in their "My Chats" list.

| Field | Type | Description |
|-------|------|-------------|
| dialog_id | UUID | Reference to dialog |
| user_id | UUID | External user identifier |
| display_name | STRING | User's display name |
| company | STRING | User's company |
| email | STRING | Contact email (optional) |
| phone | STRING | Contact phone (optional) |
| joined_at | TIMESTAMP | When user joined |
| joined_as | ENUM | "creator" or "member" |
| notifications_enabled | BOOLEAN | Whether notifications are on |
| last_read_msg_id | UUID | Last read message (nullable) |
| unread_count | INTEGER | Number of unread messages |
| is_archived | BOOLEAN | Archived by this user |
| is_pinned | BOOLEAN | Pinned by this user |

- **user_id** is an external identifier from your system (not managed by MTChat)
- **display_name** and **company** are set when the user joins
- **unread_count** is tracked per participant
- **is_archived** and **is_pinned** are per-user states

### Access Scope

Defines rules for potential participants -- users who can discover and join the dialog.

| Field | Type | Description |
|-------|------|-------------|
| dialog_id | UUID | Reference to dialog |
| tenant_uid | STRING | Tenant identifier |
| scope_level1 | STRING[] | Departments |
| scope_level2 | STRING[] | Permissions/roles |

A dialog can have multiple access scopes. See [Scope Matching](scope-matching.md) for details on the matching algorithm.

### Message

| Field | Type | Description |
|-------|------|-------------|
| id | UUID (v7) | Time-ordered unique identifier |
| dialog_id | UUID | Reference to dialog |
| sender_id | UUID | Sender (nullable for system messages) |
| message_type | ENUM | "user" or "system" |
| content | STRING | Message content (HTML) |
| reply_to_id | UUID | Referenced message (nullable) |
| is_edited | BOOLEAN | Whether message was edited |
| is_deleted | BOOLEAN | Whether message was deleted |
| created_at | TIMESTAMP | Creation timestamp |
| updated_at | TIMESTAMP | Last update timestamp |

- User messages contain sanitized HTML (allowed tags: `p`, `br`, `strong`, `em`, `u`, `s`, `a`, `ul`, `ol`, `li`, `blockquote`, `code`, `pre`, `span`)
- System messages (joins, leaves, creation) have `message_type = "system"` and `sender_id = NULL`
- System messages store JSON content for i18n rendering on the frontend
- Edited messages retain `is_edited = true`; deleted messages retain `is_deleted = true`

### Attachment

| Field | Type | Description |
|-------|------|-------------|
| id | UUID (v7) | Unique identifier |
| message_id | UUID | Reference to message |
| s3_key | STRING | S3 object key |
| filename | STRING | Original filename |
| content_type | STRING | MIME type |
| size | BIGINT | File size in bytes |
| created_at | TIMESTAMP | Upload timestamp |

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
