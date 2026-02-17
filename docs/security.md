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

### Chat API (Host App Responsibility)

The Chat API identifies users via the `user_id` query parameter. **MTChat does not authenticate end users directly.** This is by design:

- Your host application manages user authentication (login, sessions, JWT)
- Your host application passes the authenticated `user_id` to the MTChat SDK
- MTChat trusts the `user_id` provided by the SDK

This architecture means your host application is responsible for ensuring that only authenticated users can access the chat SDK with their correct `user_id`.

!!! tip "Production recommendation"
    In production, place MTChat behind an API gateway or reverse proxy that validates user tokens before forwarding requests. This ensures that unauthenticated requests never reach MTChat.

## Access Control

### Participant-based Access

- Only **direct participants** can read messages, send messages, and view participant details
- Non-participants receive `403 Forbidden` on all message-related endpoints
- Potential participants (matched by scope) can see dialog metadata but not messages until they join

### Scope-based Visibility

- The "Available" dialog list is filtered by [scope matching](guide/scope-matching.md)
- Users only see dialogs matching their `tenant_uid`, `scope_level1`, and `scope_level2`
- Scope configuration is passed via the `X-Scope-Config` header (base64-encoded JSON)

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
- [ ] Set a strong `WEBHOOK_SECRET` for webhook signature verification
- [ ] Place MTChat behind a reverse proxy with TLS (HTTPS)
- [ ] Restrict CORS to your application's domains
- [ ] Ensure your host application authenticates users before providing the chat SDK
- [ ] Use the `user_id` from your authentication system (not user-supplied input)
- [ ] Set appropriate `RUST_LOG` level for production (avoid `debug`)
- [ ] Configure `S3_PUBLIC_ENDPOINT` to use HTTPS in production
- [ ] Monitor the `/health` and `/health/ready` endpoints
- [ ] Consider rate limiting at the reverse proxy level
- [ ] Review S3 bucket permissions (no public access)

## Future Enhancements

The following security features are planned for future releases:

- **JWT authentication** for the Chat API (validate user tokens server-side)
- **WebSocket subscription filtering** (users only receive events for their dialogs)
- **Built-in rate limiting** per user and per endpoint
- **CSRF protection** for non-API endpoints
