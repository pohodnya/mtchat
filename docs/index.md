# MTChat

**Embeddable chat service for B2B/B2C platforms.**

MTChat is a microservice-based chat system designed to be embedded into business applications. It binds conversations to your domain objects (orders, tenders, routes, etc.) and manages participant access through a flexible scope-matching system.

## Key Features

- **Object-bound dialogs** -- chats are linked to business objects in your system
- **Multiple chats per object** -- create as many conversations as needed for a single entity
- **Scope-based access control** -- define who can see and join chats using tenant + department + role matching
- **Direct + potential participants** -- explicitly added users get notifications; matching users can discover and join
- **Rich messaging** -- formatting (bold, italic, lists, links), file attachments, replies, editing, deletion
- **Real-time** -- WebSocket-based instant messaging with online presence tracking
- **Vue SDK** -- drop-in Vue 3 component with full and inline display modes
- **Themeable** -- light/dark themes with CSS variable customization
- **Internationalization** -- Russian, English, and Chinese out of the box

## Architecture

```
┌───────────────────────────────────────────────────────────┐
│                  Host Application                         │
│  ┌─────────────┐              ┌──────────────────────┐    │
│  │  Frontend   │              │  Backend             │    │
│  │ ┌────────┐  │              │  - Create dialogs    │    │
│  │ │ MTChat │  │              │  - Manage members    │    │
│  │ │Vue SDK │  │              │  - Handle webhooks   │    │
│  │ └───┬────┘  │              └───────────┬──────────┘    │
│  └─────┼───────┘                          │               │
└────────┼──────────────────────────────────┼───────────────┘
         │                                  │
         │ Chat API                         │ Management API
         │                                  │ (Admin Token)
         ▼                                  ▼
┌───────────────────────────────────────────────────────────┐
│                    MTChat Backend                         │
│  ┌──────────────┐  ┌────────────────┐  ┌────────────┐     │
│  │   Chat API   │  │ Management API │  │  Webhooks  │     │
│  └──────────────┘  └────────────────┘  └────────────┘     │
│                                                           │
│            PostgreSQL + Redis + S3 (MinIO)                │
└───────────────────────────────────────────────────────────┘
```

## Quick Links

<div class="grid cards" markdown>

-   :material-rocket-launch: **Getting Started**

    ---

    Install MTChat and run it in under 5 minutes.

    [:octicons-arrow-right-24: Installation](getting-started/installation.md)
    [:octicons-arrow-right-24: Quick Start](getting-started/quickstart.md)

-   :material-book-open-variant: **Guide**

    ---

    Understand the architecture, data model, and access control.

    [:octicons-arrow-right-24: Architecture](guide/architecture.md)
    [:octicons-arrow-right-24: Data Model](guide/data-model.md)
    [:octicons-arrow-right-24: Scope Matching](guide/scope-matching.md)

-   :material-api: **API Reference**

    ---

    Management API, Chat API, WebSocket events, webhooks, file upload.

    [:octicons-arrow-right-24: Management API](api/management.md)
    [:octicons-arrow-right-24: Chat API](api/chat.md)
    [:octicons-arrow-right-24: WebSocket](api/websocket.md)

-   :material-vuejs: **Vue SDK**

    ---

    Drop-in Vue 3 component with full and inline modes.

    [:octicons-arrow-right-24: Components](sdk/components.md)
    [:octicons-arrow-right-24: Configuration](sdk/configuration.md)
    [:octicons-arrow-right-24: Theming](sdk/theming.md)

</div>

## Tech Stack

| Component | Technology |
|-----------|------------|
| Backend | Rust (axum, sqlx, tokio) |
| SDK | TypeScript + Vue.js 3 |
| Database | PostgreSQL 17 |
| Cache / PubSub | Redis 7 |
| Object Storage | S3-compatible (MinIO) |
| Job Queue | apalis (Redis backend) |

## License

MTChat is open-source software licensed under the [MIT License](https://github.com/nicenemo/mtchat/blob/master/LICENSE).
