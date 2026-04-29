# Security

This page describes MTChat's security model, best practices, and recommendations for production deployments.

## Authentication Model

MTChat has two separate authentication mechanisms:

### Management API (Admin Token)

The Management API is protected by a static bearer token:

```
Authorization: Bearer <ADMIN_API_TOKEN>
```

- Set via the `ADMIN_API_TOKEN` environment variable
- Used for server-to-server communication between your backend and MTChat
- Constant-time token comparison prevents timing attacks
- Token is read once at startup (not from env on every request)

!!! warning "Keep the admin token secret"
    The admin token grants full access to create/delete dialogs and manage participants. Never expose it to frontend clients.

### Chat API (JWT Authentication)

The Chat API supports optional JWT authentication. When enabled (`JWT_AUTH_ENABLED=true`), MTChat validates user tokens server-side:

**How it works:**

- REST API: Token passed in `Authorization: Bearer <token>` header
- WebSocket: Token passed as `?token=<token>` query parameter
- User ID extracted from the JWT claim configured via `JWT_USER_ID_CLAIM` (default: `sub`)
- Signature validated using HS256 with `JWT_SECRET`
- Expiration (`exp`) is NOT validated -- token lifetime is managed by your host application

**Token format:**

```json
{
  "sub": "user-id-here",
  "iat": 1234567890
}
```

**SDK configuration:**

```typescript
const config: MTChatConfig = {
  baseUrl: 'https://chat.example.com',
  userId: user.id,
  token: jwtToken,  // JWT from your auth system
  // ...
}
```

!!! tip "Production recommendation"
    Enable JWT authentication in production (`JWT_AUTH_ENABLED=true`) to ensure that only authenticated users can access the chat. Set a strong `JWT_SECRET` (minimum 32 characters).

**Legacy mode (JWT disabled):**

When `JWT_AUTH_ENABLED=false` (default), the Chat API identifies users via the `user_id` query parameter. In this mode, your host application is responsible for ensuring that only authenticated users can access the chat SDK with their correct `user_id`.

## Access Control

### Participant-based Access

- Only **direct participants** can read messages, send messages, and view participant details
- Non-participants receive `403 Forbidden` on all message-related endpoints
- Potential participants (matched by scope) can see dialog metadata but not messages until they join

### Scope-based Visibility

- The "Available" dialog list is filtered by [scope matching](guide/scope-matching.md)
- Users only see dialogs matching their `scope_level0`, `scope_level1`, and `scope_level2`
- Scope configuration is passed via the `X-Scope-Config` header (base64-encoded JSON)

## Input Validation

All text inputs are validated for length to prevent abuse:

| Field | Max Length | Description |
|-------|------------|-------------|
| Message content | 50,000 chars | HTML message body |
| Dialog title | 500 chars | Dialog title |
| Display name | 200 chars | Participant display name |
| Company | 200 chars | Company name |
| Email | 254 chars | Contact email |
| Phone | 50 chars | Contact phone |
| Object ID | 255 chars | External object identifier |
| User ID | 255 chars | External user identifier |

Requests exceeding these limits return `400 Bad Request` with error code `InvalidInput`.

### S3 Key Validation

File attachment S3 keys are validated to prevent path traversal attacks:

- Blocks `..` sequences and URL-encoded variants (`%2e%2e`)
- Blocks null byte injection (`%00`)
- Verifies file belongs to the correct dialog

## Input Sanitization

### HTML Content

Message content (HTML from the Tiptap editor) is sanitized server-side using the [ammonia](https://docs.rs/ammonia) crate:

**Allowed tags:** `p`, `br`, `strong`, `em`, `u`, `s`, `a`, `ul`, `ol`, `li`, `blockquote`, `code`, `pre`, `span`

**Removed:** `<script>`, event handlers (`onclick`, `onerror`, etc.), `javascript:` URLs, iframes, forms, and all other potentially dangerous HTML.

### Client-side Sanitization

The Vue SDK uses [DOMPurify](https://github.com/cure53/DOMPurify) for client-side sanitization before rendering HTML with `v-html`. This provides defense-in-depth against XSS.

## Webhook Security

Outgoing webhooks are signed with HMAC-SHA256:

- The `X-Webhook-Signature` header contains `sha256=<hex-digest>`
- The digest is computed over the raw request body using `WEBHOOK_SECRET`
- Constant-time comparison is used internally
- Always verify the signature on your webhook endpoint

See [Webhooks](api/webhooks.md) for verification examples.

## File Upload Security

- File types are validated against an allowlist of MIME types
- File size is limited to 100 MB
- Files are stored in S3 with unique UUIDv7 keys (no user-controlled paths)
- Download URLs are presigned and temporary
- S3 bucket is configured with `anonymous set none` (no public access)

## CORS

MTChat is designed to be embedded cross-origin (the SDK runs on your domain, the API runs on the MTChat domain). The default configuration allows all origins:

```
Access-Control-Allow-Origin: *
```

!!! warning "Production recommendation"
    For production deployments, restrict CORS to your application's domains using a reverse proxy or by configuring the `allowed_origins` setting.

## Production Checklist

- [ ] Set a strong, unique `ADMIN_API_TOKEN` (minimum 32 characters)
- [ ] Enable JWT authentication (`JWT_AUTH_ENABLED=true`) and set a strong `JWT_SECRET` (minimum 32 characters)
- [ ] Set a strong `WEBHOOK_SECRET` for webhook signature verification
- [ ] Place MTChat behind a reverse proxy with TLS (HTTPS)
- [ ] Restrict CORS to your application's domains
- [ ] Generate JWT tokens server-side and pass them to the SDK via the `token` config property
- [ ] Set appropriate `RUST_LOG` level for production (avoid `debug`)
- [ ] Configure `S3_PUBLIC_ENDPOINT` to use HTTPS in production
- [ ] Monitor the `/health` and `/health/ready` endpoints
- [ ] Consider rate limiting at the reverse proxy level
- [ ] Review S3 bucket permissions (no public access)

## Future Enhancements

The following security features are planned for future releases:

- **WebSocket subscription filtering** (users only receive events for their dialogs)
- **Built-in rate limiting** per user and per endpoint
- **CSRF protection** for non-API endpoints
