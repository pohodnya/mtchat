# Installation

## Backend

### Docker Compose (Recommended)

The easiest way to run MTChat is with Docker Compose, which starts all required services:

```bash
git clone https://github.com/nicenemo/mtchat.git
cd mtchat
docker-compose up -d
```

This starts PostgreSQL, Redis, MinIO, the API server, and a demo application.

| Service | URL | Description |
|---------|-----|-------------|
| API | `http://localhost:8080` | MTChat backend |
| Demo App | `http://localhost` | Example application |
| PostgreSQL | `localhost:5432` | Database |
| Redis | `localhost:6379` | Cache, PubSub, Jobs |
| MinIO | `http://localhost:9000` | S3-compatible storage |
| MinIO Console | `http://localhost:9001` | Storage admin panel |

#### Graceful Degradation

MTChat works with only PostgreSQL. Other services are optional:

| Service | Required | What it enables |
|---------|----------|-----------------|
| PostgreSQL | **Yes** | Core data storage |
| Redis | No | Online status, job queue, smart notifications, auto-archive |
| MinIO / S3 | No | File attachments (upload/download) |
| Webhook URL | No | Outgoing event notifications to your backend |

### Docker (Standalone)

Run only the API container with an external PostgreSQL:

```bash
docker run -d \
  --name mtchat-api \
  -p 8080:8080 \
  -e DATABASE_URL=postgres://user:pass@your-db:5432/mtchat \
  -e ADMIN_API_TOKEN=your-admin-token \
  ghcr.io/nicenemo/mtchat:latest
```

### Helm Chart (Kubernetes)

A Helm chart is available for Kubernetes deployments:

```bash
helm install mtchat ./deploy/helm/mtchat \
  --set postgresql.auth.password=your-password \
  --set config.adminApiToken=your-admin-token
```

See `deploy/helm/mtchat/values.yaml` for all configuration options.

### Building from Source

Requirements: Rust 1.75+, PostgreSQL 17

```bash
cd mtchat-rust
cargo build --release

# Run migrations automatically on startup
DATABASE_URL=postgres://user:pass@localhost:5432/mtchat \
ADMIN_API_TOKEN=your-token \
  ./target/release/multitenancy-chat-api
```

The server runs database migrations automatically on startup.

---

## Vue SDK

### npm

```bash
npm install @mtchat/vue
```

### Peer Dependencies

The SDK requires Vue 3.4+:

```bash
npm install vue@^3.4
```

### Importing

```ts
// Component (styles are bundled automatically)
import { MTChat } from '@mtchat/vue'

// Optional: composables for headless usage
import { useChat, useFileUpload } from '@mtchat/vue'

// Optional: types
import type { MTChatConfig, MTChatProps, Dialog, Message } from '@mtchat/vue'
```

### CDN (UMD)

For non-bundler environments:

```html
<script src="https://unpkg.com/vue@3/dist/vue.global.prod.js"></script>
<script src="https://unpkg.com/@mtchat/vue/dist/mtchat-vue.umd.cjs"></script>

<script>
const { MTChat } = window.MtchatVue
// Use as a component
</script>
```
