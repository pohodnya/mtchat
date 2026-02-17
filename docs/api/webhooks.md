# Webhooks

MTChat sends outgoing HTTP webhooks to your backend when important events occur. This allows your application to react to chat events (e.g., send push notifications, update business logic).

## Configuration

Set the webhook URL and signing secret via environment variables:

```bash
WEBHOOK_URL=https://your-app.com/webhooks/mtchat
WEBHOOK_SECRET=your-signing-secret
```

If `WEBHOOK_URL` is not set, webhooks are disabled.

## Request Format

All webhook events are sent as HTTP POST requests with a JSON body:

```
POST https://your-app.com/webhooks/mtchat
Content-Type: application/json
X-Webhook-Signature: sha256=<hmac-hex-digest>
X-Webhook-Event: message.new
```

### Headers

| Header | Description |
|--------|-------------|
| `Content-Type` | Always `application/json` |
| `X-Webhook-Signature` | HMAC-SHA256 signature of the request body |
| `X-Webhook-Event` | Event type (e.g., `message.new`) |

### Signature Verification

The `X-Webhook-Signature` header contains an HMAC-SHA256 hex digest of the request body, signed with `WEBHOOK_SECRET`. Verify it to ensure the request is authentic:

```python
# Python example
import hmac, hashlib

def verify_webhook(body: bytes, signature: str, secret: str) -> bool:
    expected = hmac.new(
        secret.encode(), body, hashlib.sha256
    ).hexdigest()
    return hmac.compare_digest(f"sha256={expected}", signature)
```

```javascript
// Node.js example
const crypto = require('crypto');

function verifyWebhook(body, signature, secret) {
  const expected = crypto
    .createHmac('sha256', secret)
    .update(body)
    .digest('hex');
  return crypto.timingSafeEqual(
    Buffer.from(`sha256=${expected}`),
    Buffer.from(signature)
  );
}
```

!!! warning
    Always use constant-time comparison to prevent timing attacks.

## Event Envelope

All events share a common envelope structure:

```json
{
  "id": "019481e5-...",
  "type": "message_new",
  "timestamp": "2026-02-17T12:10:00Z",
  "payload": { ... }
}
```

| Field | Type | Description |
|-------|------|-------------|
| `id` | UUID | Unique event ID |
| `type` | string | Event type identifier |
| `timestamp` | datetime | When the event occurred |
| `payload` | object | Event-specific data |

## Event Types

### message.new

A new message was sent in a dialog.

```json
{
  "id": "019481e5-...",
  "type": "message_new",
  "timestamp": "2026-02-17T12:10:00Z",
  "payload": {
    "dialog_id": "019481a2-...",
    "object_id": "550e8400-...",
    "object_type": "order",
    "message": {
      "id": "019481b3-...",
      "sender_id": "11111111-...",
      "content": "<p>Hello!</p>",
      "reply_to": null,
      "created_at": "2026-02-17T12:10:00Z",
      "message_type": "user"
    }
  }
}
```

### participant.joined

A user joined a dialog.

```json
{
  "id": "019481e6-...",
  "type": "participant_joined",
  "timestamp": "2026-02-17T12:05:00Z",
  "payload": {
    "dialog_id": "019481a2-...",
    "object_id": "550e8400-...",
    "object_type": "order",
    "user_id": "33333333-...",
    "joined_as": "member",
    "joined_at": "2026-02-17T12:05:00Z"
  }
}
```

### participant.left

A user left a dialog.

```json
{
  "id": "019481e7-...",
  "type": "participant_left",
  "timestamp": "2026-02-17T12:20:00Z",
  "payload": {
    "dialog_id": "019481a2-...",
    "object_id": "550e8400-...",
    "object_type": "order",
    "user_id": "33333333-...",
    "left_at": "2026-02-17T12:20:00Z"
  }
}
```

### notification.pending

Sent when a message has not been read by a recipient after a configurable delay (default: 30 seconds). Your application should send a push notification or email to the recipient.

```json
{
  "id": "019481e8-...",
  "type": "notification_pending",
  "timestamp": "2026-02-17T12:10:30Z",
  "payload": {
    "dialog_id": "019481a2-...",
    "object_id": "550e8400-...",
    "object_type": "order",
    "recipient_id": "22222222-...",
    "message": {
      "id": "019481b3-...",
      "sender_id": "11111111-...",
      "content": "<p>Hello!</p>",
      "reply_to": null,
      "created_at": "2026-02-17T12:10:00Z",
      "message_type": "user"
    }
  }
}
```

**Smart notification behavior:**

- Notifications are delayed (default 30 seconds, configurable via `NOTIFICATION_DELAY_SECS`)
- If the message is read before the delay expires, no notification is sent
- Multiple messages to the same recipient within the delay window are debounced into a single notification
- Notifications are skipped if the user has disabled notifications for that dialog

## Retry Policy

Failed webhook deliveries are retried with exponential backoff:

- Maximum retries: 3
- Initial delay: 1 second
- Backoff multiplier: 2x
- Request timeout: 10 seconds

Your endpoint should return a 2xx status code to acknowledge receipt.

## Webhook Receiver Example

```javascript
// Express.js webhook handler
const crypto = require('crypto');

app.post('/webhooks/mtchat', express.raw({ type: 'application/json' }), (req, res) => {
  // Verify signature
  const signature = req.headers['x-webhook-signature'];
  const expected = `sha256=${crypto
    .createHmac('sha256', process.env.WEBHOOK_SECRET)
    .update(req.body)
    .digest('hex')}`;

  if (!crypto.timingSafeEqual(Buffer.from(expected), Buffer.from(signature))) {
    return res.status(401).send('Invalid signature');
  }

  const event = JSON.parse(req.body);
  const eventType = req.headers['x-webhook-event'];

  switch (eventType) {
    case 'message.new':
      console.log('New message in dialog', event.payload.dialog_id);
      break;
    case 'notification.pending':
      // Send push notification to event.payload.recipient_id
      sendPushNotification(event.payload.recipient_id, event.payload.message);
      break;
  }

  res.status(200).send('OK');
});
```
