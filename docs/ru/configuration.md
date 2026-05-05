# Конфигурация

MTChat настраивается через переменные окружения. У `DATABASE_URL` есть локальное значение по умолчанию, но в развернутых окружениях его нужно задавать явно. `ADMIN_API_TOKEN` нужно задавать вне локальной разработки; если он не указан, Management API запускается без защиты.

## Основные переменные

| Переменная | Описание | Пример |
|------------|----------|--------|
| `DATABASE_URL` | Строка подключения PostgreSQL; при отсутствии используется локальный PostgreSQL | `postgres://user:pass@localhost:5432/mtchat` |
| `ADMIN_API_TOKEN` | Токен для аутентификации Management API; обязателен для защищенных окружений | `your-secure-admin-token` |

## Сервер

| Переменная | По умолчанию | Описание |
|------------|--------------|----------|
| `PORT` | `8080` | Порт HTTP-сервера |
| `RUST_LOG` | `info` | Уровень логирования |

## Пул базы данных

| Переменная | По умолчанию | Описание |
|------------|--------------|----------|
| `DATABASE_MAX_CONNECTIONS` | `20` | Максимальный размер пула PostgreSQL |
| `DATABASE_MIN_CONNECTIONS` | `5` | Минимальный размер пула PostgreSQL |
| `DATABASE_ACQUIRE_TIMEOUT_SECS` | `30` | Таймаут получения соединения |
| `DATABASE_IDLE_TIMEOUT_SECS` | `600` | Таймаут простоя соединения |
| `DATABASE_MAX_LIFETIME_SECS` | `1800` | Максимальное время жизни соединения |

## Redis (опционально)

Включает онлайн-присутствие, очередь задач и умные уведомления.

| Переменная | По умолчанию | Описание |
|------------|--------------|----------|
| `REDIS_URL` | -- | URL подключения к Redis |

## S3 / MinIO (опционально)

Включает загрузку и скачивание файловых вложений.

| Переменная | По умолчанию | Описание |
|------------|--------------|----------|
| `S3_ENDPOINT` | -- | URL S3/MinIO сервера |
| `S3_PUBLIC_ENDPOINT` | `S3_ENDPOINT` | Публичный URL S3 для presigned-ссылок |
| `S3_BUCKET` | -- | Имя бакета S3 |
| `S3_ACCESS_KEY_ID` | -- | Ключ доступа S3 |
| `S3_SECRET_ACCESS_KEY` | -- | Секретный ключ S3 |
| `S3_REGION` | `us-east-1` | Регион S3 |
| `S3_PRESIGN_UPLOAD_EXPIRY` | `300` | Время жизни upload URL в секундах |
| `S3_PRESIGN_DOWNLOAD_EXPIRY` | `3600` | Время жизни download URL в секундах |

!!! tip
    `S3_PUBLIC_ENDPOINT` -- URL, по которому браузеры обращаются к S3. В локальной разработке с MinIO это обычно `http://localhost:9000`, а `S3_ENDPOINT` -- внутренний URL Docker-сети `http://minio:9000`.

## Вебхуки (опционально)

| Переменная | По умолчанию | Описание |
|------------|--------------|----------|
| `WEBHOOK_URL` | -- | URL вашего вебхук-эндпоинта |
| `WEBHOOK_SECRET` | -- | Секрет для HMAC-SHA256 подписи |

## Фоновые задачи

| Переменная | По умолчанию | Описание |
|------------|--------------|----------|
| `NOTIFICATION_CONCURRENCY` | `4` | Количество параллельных воркеров |
| `ARCHIVE_CRON` | `0 */5 * * * *` | Расписание проверки авто-архивации |
| `ARCHIVE_AFTER_SECS` | `259200` | Секунды неактивности до авто-архивации (3 дня) |

Задачи уведомлений сейчас используют короткую фиксированную задержку перед проверкой, было ли сообщение прочитано.

## Rate limiting

Встроенный rate limiting по умолчанию выключен.

| Переменная | По умолчанию | Описание |
|------------|--------------|----------|
| `RATE_LIMIT_ENABLED` | `false` | Включить token-bucket rate limiter |
| `RATE_LIMIT_RPS` | `100` | Скорость пополнения лимита, запросов в секунду |
| `RATE_LIMIT_BURST` | `50` | Размер burst-окна |

## CORS

Настройка CORS для API.

| Переменная | По умолчанию | Описание |
|------------|--------------|----------|
| `CORS_ALLOWED_ORIGINS` | `*` | Разрешённые origins через запятую или `*` для всех |
| `CORS_ALLOWED_METHODS` | `GET,POST,PUT,DELETE,OPTIONS` | Разрешённые HTTP-методы |
| `CORS_ALLOWED_HEADERS` | `*` | Разрешённые заголовки или `*` для всех |
| `CORS_ALLOW_CREDENTIALS` | `false` | Разрешить credentials (`true`/`false`) |
| `CORS_MAX_AGE` | `3600` | Время кэширования preflight (секунды) |

**Примеры:**

```bash
# Разработка (по умолчанию - открыто)
# Настройка не требуется

# Продакшен (конкретные домены)
CORS_ALLOWED_ORIGINS="https://app.example.com,https://admin.example.com"
CORS_ALLOW_CREDENTIALS="true"
```

## Аутентификация

### Management API

| Переменная | По умолчанию | Описание |
|------------|--------------|----------|
| `ADMIN_API_TOKEN` | -- | Bearer-токен для Management API; не указывайте только в локальной разработке |

### Chat API (JWT)

Опциональная JWT-аутентификация для Chat API. При включении валидирует подпись токена (HS256) без проверки срока действия.

| Переменная | По умолчанию | Описание |
|------------|--------------|----------|
| `JWT_AUTH_ENABLED` | `false` | Включить JWT-аутентификацию для Chat API |
| `JWT_SECRET` | -- | Секретный ключ для HS256 (обязателен если JWT включён) |
| `JWT_USER_ID_CLAIM` | `sub` | Имя claim, из которого читается ID пользователя |

**Как работает:**

- REST API: токен передаётся в заголовке `Authorization: Bearer <token>`
- WebSocket: токен передаётся как query-параметр `?token=<token>`
- ID пользователя извлекается из claim, указанного в `JWT_USER_ID_CLAIM` (по умолчанию `sub`)
- Числовые значения claim приводятся к строке, чтобы соответствовать `String`-идентификаторам MTChat
- При выключении используется `?user_id=<id>` (legacy-режим)

**Формат токена (HS256), стандартный claim:**

```json
{
  "sub": "user-id-here",
  "iat": 1234567890
}
```

**Кастомный claim** — если хост-приложение кладёт ID под нестандартным именем:

```bash
JWT_USER_ID_CLAIM=user_id
```

```json
{
  "user_id": "user-id-here",
  "iat": 1234567890
}
```

!!! tip "Переиспользование токена"
    MTChat не проверяет срок действия токена (`exp`). Токен переиспользуется из вашей системы аутентификации.

## Мониторинг

| Переменная | По умолчанию | Описание |
|------------|--------------|----------|
| `SENTRY_DSN` | -- | DSN для Sentry error tracking |

## Health Checks

| Эндпоинт | Описание |
|----------|----------|
| `GET /health` | Проверка работоспособности |
| `GET /health/ready` | Проверка готовности (верифицирует подключение к БД) |

## Docker Compose Example

```yaml
services:
  api:
    image: pohodnya/mtchat:latest
    environment:
      DATABASE_URL: postgres://postgres:postgres@postgres:5432/multitenancy_chat
      ADMIN_API_TOKEN: ${ADMIN_API_TOKEN}
      REDIS_URL: redis://redis:6379
      S3_ENDPOINT: http://minio:9000
      S3_PUBLIC_ENDPOINT: http://localhost:9000
      S3_BUCKET: mtchat-attachments
      S3_ACCESS_KEY_ID: minioadmin
      S3_SECRET_ACCESS_KEY: minioadmin
      S3_REGION: us-east-1
      WEBHOOK_URL: https://your-app.com/webhooks/mtchat
      WEBHOOK_SECRET: your-webhook-secret
      NOTIFICATION_CONCURRENCY: 4
      ARCHIVE_CRON: "0 */5 * * * *"
      ARCHIVE_AFTER_SECS: 259200
      RUST_LOG: multitenancy_chat_api=info,tower_http=info
    ports:
      - "8080:8080"
```

## Graceful Degradation

MTChat работает с минимальной инфраструктурой. Только PostgreSQL обязателен:

| Функция | Требует |
|---------|---------|
| Базовый обмен сообщениями | PostgreSQL |
| Онлайн-статус | PostgreSQL + Redis |
| Файловые вложения | PostgreSQL + S3 |
| Умные уведомления | PostgreSQL + Redis + Webhook URL |
| Авто-архивация | PostgreSQL + Redis |

Все опциональные функции деградируют gracefully при отсутствии зависимостей.
