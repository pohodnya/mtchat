# Management API

The Management API is used by your backend to create and manage dialogs. All requests require an admin token in the `Authorization` header.

## Authentication

```
Authorization: Bearer <ADMIN_API_TOKEN>
```

The admin token is configured via the `ADMIN_API_TOKEN` environment variable on the MTChat server.

---

## Create Dialog

Creates a new dialog with participants and access scopes.

```
POST /api/v1/management/dialogs
```

### Request Body

```json
{
  "object_id": "550e8400-e29b-41d4-a716-446655440000",
  "object_type": "order",
  "title": "Order #1234 Discussion",
  "object_url": "https://app.example.com/orders/1234",
  "participants": [
    {
      "user_id": "11111111-1111-1111-1111-111111111111",
      "display_name": "Alice",
      "company": "Acme Inc",
      "email": "alice@acme.com",
      "phone": "+1234567890"
    }
  ],
  "access_scopes": [
    {
      "tenant_uid": "22222222-2222-2222-2222-222222222222",
      "scope_level1": ["logistics", "sales"],
      "scope_level2": ["manager", "admin"]
    }
  ]
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `object_id` | UUID | Yes | ID of the business object this dialog belongs to |
| `object_type` | string | Yes | Type of the business object (e.g., "order", "tender") |
| `title` | string | No | Dialog title (displayed in chat list) |
| `object_url` | string | No | URL linking back to the object in your application |
| `participants` | array | Yes | Initial participants (at least one recommended) |
| `participants[].user_id` | UUID | Yes | User ID from your system |
| `participants[].display_name` | string | Yes | Display name shown in chat |
| `participants[].company` | string | No | Company name |
| `participants[].email` | string | No | Contact email |
| `participants[].phone` | string | No | Contact phone |
| `access_scopes` | array | No | Scope rules for potential participants |
| `access_scopes[].tenant_uid` | UUID | Yes | Tenant/organization ID |
| `access_scopes[].scope_level1` | string[] | No | First scope level (e.g., departments). Empty = match any. |
| `access_scopes[].scope_level2` | string[] | No | Second scope level (e.g., roles). Empty = match any. |

### Response

```json
{
  "data": {
    "id": "019481a2-...",
    "object_id": "550e8400-...",
    "object_type": "order",
    "title": "Order #1234 Discussion",
    "object_url": "https://app.example.com/orders/1234",
    "created_by": "11111111-...",
    "created_at": "2026-02-17T12:00:00Z"
  }
}
```

!!! note
    Multiple dialogs can be created for the same `object_id` / `object_type` combination.

---

## Get Dialog

Retrieves a dialog with its participants and access scopes.

```
GET /api/v1/management/dialogs/{id}
```

### Response

```json
{
  "data": {
    "id": "019481a2-...",
    "object_id": "550e8400-...",
    "object_type": "order",
    "title": "Order #1234 Discussion",
    "created_by": "11111111-...",
    "created_at": "2026-02-17T12:00:00Z",
    "participants": [
      {
        "user_id": "11111111-...",
        "display_name": "Alice",
        "company": "Acme Inc",
        "joined_as": "creator",
        "joined_at": "2026-02-17T12:00:00Z"
      }
    ],
    "access_scopes": [
      {
        "tenant_uid": "22222222-...",
        "scope_level1": ["logistics"],
        "scope_level2": ["manager", "admin"]
      }
    ]
  }
}
```

---

## Delete Dialog

Deletes a dialog and all its data (participants, messages, attachments, scopes).

```
DELETE /api/v1/management/dialogs/{id}
```

### Response

```json
{
  "data": null
}
```

---

## Add Participant

Adds a participant to an existing dialog.

```
POST /api/v1/management/dialogs/{id}/participants
```

### Request Body

```json
{
  "user_id": "33333333-3333-3333-3333-333333333333",
  "display_name": "Bob",
  "company": "Partner Inc",
  "email": "bob@partner.com",
  "phone": "+0987654321"
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `user_id` | UUID | Yes | User ID from your system |
| `display_name` | string | Yes | Display name shown in chat |
| `company` | string | No | Company name |
| `email` | string | No | Contact email |
| `phone` | string | No | Contact phone |

The participant's `joined_as` is set to `"member"` when added via the Management API.

### Response

```json
{
  "data": {
    "user_id": "33333333-...",
    "display_name": "Bob",
    "company": "Partner Inc",
    "joined_as": "member",
    "joined_at": "2026-02-17T12:05:00Z"
  }
}
```

---

## Remove Participant

Removes a participant from a dialog.

```
DELETE /api/v1/management/dialogs/{id}/participants/{user_id}
```

### Response

```json
{
  "data": null
}
```

---

## Update Access Scopes

Replaces all access scopes for a dialog.

```
PUT /api/v1/management/dialogs/{id}/access-scopes
```

### Request Body

```json
{
  "access_scopes": [
    {
      "tenant_uid": "22222222-2222-2222-2222-222222222222",
      "scope_level1": ["logistics"],
      "scope_level2": ["admin"]
    }
  ]
}
```

This **replaces** all existing scopes. To remove all scopes, send an empty array.

### Response

```json
{
  "data": null
}
```

---

## Error Responses

All errors follow a standard format:

```json
{
  "error": {
    "code": "NOT_FOUND",
    "message": "Dialog not found"
  }
}
```

| HTTP Status | Code | Description |
|-------------|------|-------------|
| 400 | `BAD_REQUEST` | Invalid request body |
| 401 | `UNAUTHORIZED` | Missing or invalid admin token |
| 404 | `NOT_FOUND` | Dialog or participant not found |
| 500 | `INTERNAL_ERROR` | Server error |
