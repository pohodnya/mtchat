# Security Policy

## Reporting a Vulnerability

If you discover a security vulnerability in MTChat, please report it responsibly.

**Do not open a public GitHub issue for security vulnerabilities.**

### How to Report

Send an email to **security@mtchat.dev** (or use [GitHub Security Advisories](https://github.com/pohodnya/mtchat/security/advisories/new)) with:

- Description of the vulnerability
- Steps to reproduce
- Potential impact
- Suggested fix (if any)

### Response Timeline

- **Acknowledgment:** within 48 hours
- **Initial assessment:** within 1 week
- **Fix and disclosure:** coordinated with reporter

## Supported Versions

| Version | Supported |
|---------|-----------|
| Latest release | Yes |
| Previous minor | Security fixes only |
| Older versions | No |

## Security Considerations

### Authentication

- **Management API** is protected by an admin token (`ADMIN_API_TOKEN` env var)
- **Chat API** uses user tokens passed via headers -- your application is responsible for token generation and validation
- Admin tokens are compared using constant-time SHA-256 digest comparison to prevent timing attacks

### Data Access

- Users can only access chats they participate in or have scope access to
- Potential participants cannot read messages until they join
- All message-related endpoints validate participant status

### Input Validation

- HTML message content is sanitized using the `ammonia` crate
- Only safe HTML tags are allowed (p, br, strong, em, a, ul, ol, li, blockquote, code, pre)
- Scripts, event handlers, and `javascript:` URLs are stripped

### File Uploads

- Files are uploaded directly to S3 via presigned URLs (never through the API server)
- File type validation is performed server-side
- Maximum file size: 100 MB
- Maximum attachments per message: 10

### Webhook Security

- Outgoing webhooks are signed with HMAC-SHA256 using the `WEBHOOK_SECRET`
- Verify the `X-Webhook-Signature` header on your webhook receiver

### Infrastructure

- The backend runs as a non-root user in Docker containers
- Database connections use connection pooling with configurable limits
- Redis is used for ephemeral data only (presence, job queue) -- no sensitive data persisted in Redis
