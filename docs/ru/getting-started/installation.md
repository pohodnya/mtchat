# Установка

## Бэкенд

### Docker Compose (рекомендуется)

Самый простой способ запустить MTChat -- Docker Compose, который стартует все необходимые сервисы:

```bash
git clone https://github.com/nicenemo/mtchat.git
cd mtchat
docker-compose up -d
```

Это запускает PostgreSQL, Redis, MinIO, API-сервер и демо-приложение.

| Сервис | URL | Описание |
|--------|-----|----------|
| API | `http://localhost:8080` | MTChat бэкенд |
| Демо-приложение | `http://localhost` | Пример приложения |
| PostgreSQL | `localhost:5432` | База данных |
| Redis | `localhost:6379` | Кэш, PubSub, задачи |
| MinIO | `http://localhost:9000` | S3-совместимое хранилище |
| MinIO Console | `http://localhost:9001` | Админка хранилища |

#### Graceful Degradation

MTChat работает только с PostgreSQL. Остальные сервисы опциональны:

| Сервис | Обязателен | Что включает |
|--------|------------|--------------|
| PostgreSQL | **Да** | Основное хранилище данных |
| Redis | Нет | Онлайн-статус, очередь задач, умные уведомления, авто-архивация |
| MinIO / S3 | Нет | Файловые вложения (загрузка/скачивание) |
| Webhook URL | Нет | Исходящие уведомления о событиях |

### Docker (отдельный контейнер)

Запуск только API-контейнера с внешним PostgreSQL:

```bash
docker run -d \
  --name mtchat-api \
  -p 8080:8080 \
  -e DATABASE_URL=postgres://user:pass@your-db:5432/mtchat \
  -e ADMIN_API_TOKEN=your-admin-token \
  ghcr.io/nicenemo/mtchat:latest
```

### Helm Chart (Kubernetes)

Для деплоя в Kubernetes доступен Helm chart:

```bash
helm install mtchat ./deploy/helm/mtchat \
  --set postgresql.auth.password=your-password \
  --set config.adminApiToken=your-admin-token
```

Все параметры описаны в `deploy/helm/mtchat/values.yaml`.

### Сборка из исходников

Требования: Rust 1.75+, PostgreSQL 17

```bash
cd mtchat-rust
cargo build --release

# Миграции запускаются автоматически при старте
DATABASE_URL=postgres://user:pass@localhost:5432/mtchat \
ADMIN_API_TOKEN=your-token \
  ./target/release/multitenancy-chat-api
```

Сервер автоматически применяет миграции базы данных при запуске.

---

## Vue SDK

### npm

```bash
npm install @mtchat/vue
```

### Зависимости

SDK требует Vue 3.4+:

```bash
npm install vue@^3.4
```

### Импорт

```ts
// Компонент
import { MTChat } from '@mtchat/vue'

// Стили (обязательно)
import '@mtchat/vue/style.css'

// Опционально: composables для кастомного UI
import { useChat, useFileUpload } from '@mtchat/vue'

// Опционально: типы
import type { MTChatConfig, MTChatProps, Dialog, Message } from '@mtchat/vue'
```

### CDN (UMD)

Для использования без сборщика:

```html
<link rel="stylesheet" href="https://unpkg.com/@mtchat/vue/dist/style.css">
<script src="https://unpkg.com/vue@3/dist/vue.global.prod.js"></script>
<script src="https://unpkg.com/@mtchat/vue/dist/mtchat-vue.umd.cjs"></script>

<script>
const { MTChat } = window.MtchatVue
// Используйте как компонент
</script>
```
