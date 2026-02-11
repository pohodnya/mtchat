# Plan 016: Apalis Job Queue с Debounce Middleware

## Обзор

Внедрение [apalis](https://github.com/geofmureithi/apalis) как production-ready job queue для отложенных задач с кастомным debounce middleware на базе tower::Service.

**Основные use cases:**
1. Smart notifications — отложенная отправка webhook с проверкой прочтения
2. Автоархивирование — архивация неактивных чатов (cron-задача)

## Архитектура

```
┌─────────────────────────────────────────────────────────────────────┐
│                         MTChat Backend                               │
│                                                                      │
│  ┌──────────────┐     ┌─────────────────────────────────────────┐   │
│  │  API Handler │────▶│              Job Producer                │   │
│  │ (send_msg)   │     │  jobs::enqueue_notification(...)        │   │
│  └──────────────┘     └─────────────────┬───────────────────────┘   │
│                                         │                            │
│                                         ▼                            │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │                      Redis (apalis-redis)                     │   │
│  │                                                               │   │
│  │   Queue: "mtchat:notifications"                               │   │
│  │   ┌─────────────────────────────────────────────────────┐    │   │
│  │   │ Job { dialog_id, user_id, message_id, run_at: +30s } │    │   │
│  │   └─────────────────────────────────────────────────────┘    │   │
│  │                                                               │   │
│  │   Debounce Set: "mtchat:debounce:notifications"              │   │
│  │   ┌─────────────────────────────────────────────────────┐    │   │
│  │   │ "dialog:123:user:456" => job_id                      │    │   │
│  │   └─────────────────────────────────────────────────────┘    │   │
│  └──────────────────────────────────────┬───────────────────────┘   │
│                                         │                            │
│                                         ▼                            │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │                    Apalis Worker                              │   │
│  │                                                               │   │
│  │   WorkerBuilder::new("notifications")                        │   │
│  │       .layer(DebounceLayer::new(redis.clone()))              │   │
│  │       .layer(RetryLayer::new(3))                             │   │
│  │       .layer(TimeoutLayer::new(Duration::from_secs(30)))     │   │
│  │       .build_fn(handle_notification)                         │   │
│  │                                                               │   │
│  └──────────────────────────────────────────────────────────────┘   │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## Зависимости

```toml
# Cargo.toml
[dependencies]
apalis = { version = "0.6", features = ["redis", "retry", "timeout", "tracing"] }
apalis-redis = "0.6"
apalis-cron = "0.6"  # для автоархивирования
tower = { version = "0.5", features = ["util"] }
```

## Структура файлов

```
mtchat-rust/src/
├── jobs/
│   ├── mod.rs                    # Экспорт модуля
│   ├── types.rs                  # Типы задач (NotificationJob, ArchiveJob)
│   ├── handlers.rs               # Обработчики задач
│   ├── producer.rs               # JobProducer для enqueue
│   ├── worker.rs                 # Конфигурация воркеров
│   └── middleware/
│       ├── mod.rs
│       └── debounce.rs           # DebounceLayer + DebounceService
├── main.rs                       # Запуск воркеров
└── ...
```

## Этап 1: Базовая инфраструктура apalis

### 1.1 Типы задач (`jobs/types.rs`)

```rust
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Задача отправки уведомления о новом сообщении
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationJob {
    pub dialog_id: Uuid,
    pub recipient_id: Uuid,
    pub message_id: Uuid,
    pub sender_id: Uuid,
    /// Ключ для debounce: "{dialog_id}:{recipient_id}"
    pub debounce_key: String,
}

impl NotificationJob {
    pub fn new(dialog_id: Uuid, recipient_id: Uuid, message_id: Uuid, sender_id: Uuid) -> Self {
        Self {
            dialog_id,
            recipient_id,
            message_id,
            sender_id,
            debounce_key: format!("{}:{}", dialog_id, recipient_id),
        }
    }
}

/// Задача автоархивирования неактивных чатов
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoArchiveJob {
    pub run_id: Uuid,
}

impl Default for AutoArchiveJob {
    fn default() -> Self {
        Self { run_id: Uuid::now_v7() }
    }
}
```

### 1.2 Producer (`jobs/producer.rs`)

```rust
use apalis::prelude::*;
use apalis_redis::RedisStorage;
use std::sync::Arc;
use std::time::Duration;

use super::types::NotificationJob;

#[derive(Clone)]
pub struct JobProducer {
    notifications: RedisStorage<NotificationJob>,
    delay: Duration,
}

impl JobProducer {
    pub fn new(notifications: RedisStorage<NotificationJob>, delay_seconds: u64) -> Self {
        Self {
            notifications,
            delay: Duration::from_secs(delay_seconds),
        }
    }

    /// Добавить задачу уведомления с задержкой
    pub async fn enqueue_notification(&self, job: NotificationJob) -> Result<(), apalis::Error> {
        self.notifications
            .clone()
            .schedule(job, self.delay)
            .await?;
        Ok(())
    }

    /// Noop producer для случаев когда очередь отключена
    pub fn noop() -> Self {
        // Создаём dummy storage который никуда не пишет
        unimplemented!("Noop producer requires mock storage")
    }
}
```

### 1.3 Обработчики (`jobs/handlers.rs`)

```rust
use apalis::prelude::*;
use sqlx::PgPool;
use std::sync::Arc;

use super::types::{NotificationJob, AutoArchiveJob};
use crate::repositories::ParticipantRepository;
use crate::webhooks::{WebhookSender, WebhookEvent};

/// Контекст для обработчиков задач
#[derive(Clone)]
pub struct JobContext {
    pub db: PgPool,
    pub participants: Arc<ParticipantRepository>,
    pub webhooks: WebhookSender,
}

/// Обработчик уведомлений
///
/// Логика:
/// 1. Проверить, прочитано ли сообщение получателем
/// 2. Если нет — отправить webhook
/// 3. Если да — пропустить (сообщение уже прочитано)
pub async fn handle_notification(
    job: NotificationJob,
    ctx: Data<JobContext>,
) -> Result<(), Error> {
    tracing::debug!(
        dialog_id = %job.dialog_id,
        recipient_id = %job.recipient_id,
        message_id = %job.message_id,
        "Processing notification job"
    );

    // Получить участника и проверить last_read_message_id
    let participant = ctx.participants
        .find(job.dialog_id, job.recipient_id)
        .await
        .map_err(|e| Error::Failed(Arc::new(e)))?;

    let should_notify = match participant {
        Some(p) => {
            // Проверяем: если last_read_message_id >= message_id, значит прочитано
            // Для точной проверки нужно сравнивать sent_at, но для простоты
            // считаем что если есть last_read и он не None — проверяем
            match p.last_read_message_id {
                Some(last_read_id) => {
                    // Сообщение ещё не прочитано если last_read_id < message_id
                    // Но UUID не сравнимы по порядку, поэтому запросим из БД
                    // Упрощённая логика: если unread_count > 0, значит не прочитано
                    p.unread_count > 0
                }
                None => true, // Никогда не читал — уведомляем
            }
        }
        None => false, // Больше не участник — не уведомляем
    };

    if should_notify {
        // Проверяем notifications_enabled
        let notifications_enabled = participant
            .map(|p| p.notifications_enabled)
            .unwrap_or(true);

        if notifications_enabled {
            tracing::info!(
                recipient_id = %job.recipient_id,
                message_id = %job.message_id,
                "Sending notification webhook"
            );

            // TODO: Загрузить dialog и message для webhook payload
            // ctx.webhooks.send(WebhookEvent::notification_pending(...)).await;
        } else {
            tracing::debug!(
                recipient_id = %job.recipient_id,
                "Notifications disabled for user, skipping"
            );
        }
    } else {
        tracing::debug!(
            recipient_id = %job.recipient_id,
            message_id = %job.message_id,
            "Message already read, skipping notification"
        );
    }

    Ok(())
}

/// Обработчик автоархивирования
///
/// Запускается по cron, архивирует чаты без активности N дней
pub async fn handle_auto_archive(
    job: AutoArchiveJob,
    ctx: Data<JobContext>,
) -> Result<(), Error> {
    tracing::info!(run_id = %job.run_id, "Running auto-archive job");

    // TODO: Реализовать логику:
    // 1. SELECT dialogs WHERE last_message_at < NOW() - INTERVAL 'N days'
    // 2. Для каждого участника установить is_archived = true
    // 3. Логировать количество заархивированных

    Ok(())
}
```

### 1.4 Тесты для handlers

```rust
// jobs/handlers_test.rs или #[cfg(test)] mod tests

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;

    /// Тест: сообщение не прочитано — webhook отправляется
    #[sqlx::test]
    async fn test_notification_sent_when_unread(pool: PgPool) {
        // Setup: создать диалог, участника с unread_count > 0
        // Act: handle_notification(job, ctx)
        // Assert: webhook был вызван (mock)
    }

    /// Тест: сообщение прочитано — webhook не отправляется
    #[sqlx::test]
    async fn test_notification_skipped_when_read(pool: PgPool) {
        // Setup: создать участника с unread_count = 0
        // Act: handle_notification(job, ctx)
        // Assert: webhook НЕ был вызван
    }

    /// Тест: notifications_enabled = false — webhook не отправляется
    #[sqlx::test]
    async fn test_notification_skipped_when_disabled(pool: PgPool) {
        // Setup: участник с notifications_enabled = false
        // Act: handle_notification(job, ctx)
        // Assert: webhook НЕ был вызван
    }

    /// Тест: участник покинул чат — webhook не отправляется
    #[sqlx::test]
    async fn test_notification_skipped_when_left(pool: PgPool) {
        // Setup: участник не существует
        // Act: handle_notification(job, ctx)
        // Assert: webhook НЕ был вызван, нет ошибки
    }
}
```

## Этап 2: Debounce Middleware

### 2.1 Концепция

Debounce работает так:
1. При enqueue задачи записываем `debounce_key -> job_id` в Redis
2. Если ключ уже существует — отменяем старую задачу, записываем новую
3. При обработке проверяем: текущий job_id совпадает с записанным?
4. Если нет — задача устарела (была заменена), пропускаем

### 2.2 DebounceLayer (`jobs/middleware/debounce.rs`)

```rust
use apalis::prelude::*;
use fred::prelude::*;
use std::sync::Arc;
use std::task::{Context, Poll};
use tower::{Layer, Service};
use std::future::Future;
use std::pin::Pin;

/// Трейт для задач с debounce ключом
pub trait Debounceable {
    fn debounce_key(&self) -> &str;
    fn job_id(&self) -> &str;
}

/// Tower Layer для debounce
#[derive(Clone)]
pub struct DebounceLayer {
    redis: Arc<RedisPool>,
    prefix: String,
    ttl_seconds: i64,
}

impl DebounceLayer {
    pub fn new(redis: Arc<RedisPool>) -> Self {
        Self {
            redis,
            prefix: "mtchat:debounce".to_string(),
            ttl_seconds: 300, // 5 минут TTL для debounce ключей
        }
    }

    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = prefix.into();
        self
    }

    pub fn with_ttl(mut self, seconds: i64) -> Self {
        self.ttl_seconds = seconds;
        self
    }

    fn redis_key(&self, debounce_key: &str) -> String {
        format!("{}:{}", self.prefix, debounce_key)
    }
}

impl<S> Layer<S> for DebounceLayer {
    type Service = DebounceService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        DebounceService {
            inner,
            redis: self.redis.clone(),
            prefix: self.prefix.clone(),
            ttl_seconds: self.ttl_seconds,
        }
    }
}

/// Tower Service с debounce логикой
#[derive(Clone)]
pub struct DebounceService<S> {
    inner: S,
    redis: Arc<RedisPool>,
    prefix: String,
    ttl_seconds: i64,
}

impl<S> DebounceService<S> {
    fn redis_key(&self, debounce_key: &str) -> String {
        format!("{}:{}", self.prefix, debounce_key)
    }
}

impl<S, J> Service<Request<J>> for DebounceService<S>
where
    S: Service<Request<J>> + Clone + Send + 'static,
    S::Future: Send,
    S::Response: Send,
    S::Error: Send,
    J: Debounceable + Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<J>) -> Self::Future {
        let mut inner = self.inner.clone();
        let redis = self.redis.clone();
        let prefix = self.prefix.clone();

        Box::pin(async move {
            let job = req.inner();
            let debounce_key = job.debounce_key();
            let current_job_id = job.job_id();
            let redis_key = format!("{}:{}", prefix, debounce_key);

            // Проверяем: текущая задача актуальна?
            let stored_job_id: Option<String> = redis.get(&redis_key).await.ok().flatten();

            match stored_job_id {
                Some(stored_id) if stored_id != current_job_id => {
                    // Задача устарела — была заменена более новой
                    tracing::debug!(
                        debounce_key = %debounce_key,
                        current_job_id = %current_job_id,
                        stored_job_id = %stored_id,
                        "Job debounced (superseded by newer job)"
                    );
                    // Возвращаем успех без выполнения
                    // Это требует что S::Response реализует Default или специальный тип
                    // Для apalis это обычно () или Result<(), Error>
                    return inner.call(req).await; // TODO: skip execution
                }
                _ => {
                    // Задача актуальна — выполняем
                    tracing::debug!(
                        debounce_key = %debounce_key,
                        "Job is current, executing"
                    );
                }
            }

            // Удаляем ключ после обработки
            let _: () = redis.del(&redis_key).await.unwrap_or_default();

            inner.call(req).await
        })
    }
}
```

### 2.3 Интеграция debounce в producer

```rust
// jobs/producer.rs

impl JobProducer {
    /// Добавить задачу с debounce
    ///
    /// Если задача с таким debounce_key уже в очереди — она будет заменена
    pub async fn enqueue_notification_debounced(
        &self,
        job: NotificationJob,
    ) -> Result<(), apalis::Error> {
        let debounce_key = job.debounce_key.clone();
        let job_id = Uuid::now_v7().to_string();

        // 1. Записываем debounce_key -> job_id в Redis
        let redis_key = format!("mtchat:debounce:notifications:{}", debounce_key);
        self.redis
            .set(&redis_key, &job_id, Some(Expiration::EX(self.delay.as_secs() as i64 + 60)), None, false)
            .await
            .map_err(|e| apalis::Error::Failed(Arc::new(e)))?;

        // 2. Создаём задачу с этим job_id
        let job_with_id = NotificationJobWithId {
            job,
            job_id,
        };

        // 3. Добавляем в очередь с задержкой
        self.notifications
            .clone()
            .schedule(job_with_id, self.delay)
            .await?;

        Ok(())
    }
}
```

### 2.4 Тесты для debounce middleware

```rust
// jobs/middleware/debounce_test.rs

#[cfg(test)]
mod tests {
    use super::*;

    /// Тест: первая задача выполняется
    #[tokio::test]
    async fn test_first_job_executes() {
        // Setup: Redis пуст
        // Act: enqueue job, process
        // Assert: job executed, debounce key deleted
    }

    /// Тест: вторая задача заменяет первую
    #[tokio::test]
    async fn test_second_job_replaces_first() {
        // Setup: enqueue job1 (delay 30s)
        // Act: enqueue job2 с тем же debounce_key
        // Assert: debounce_key теперь указывает на job2_id
    }

    /// Тест: устаревшая задача пропускается
    #[tokio::test]
    async fn test_stale_job_skipped() {
        // Setup:
        //   1. enqueue job1
        //   2. enqueue job2 (заменяет job1)
        //   3. job1 приходит на обработку
        // Act: process job1
        // Assert: job1 пропущен (debounce key != job1_id)
    }

    /// Тест: актуальная задача выполняется после замены
    #[tokio::test]
    async fn test_current_job_executes_after_replacement() {
        // Setup: enqueue job1, enqueue job2
        // Act: process job2
        // Assert: job2 executed
    }

    /// Тест: TTL на debounce ключах
    #[tokio::test]
    async fn test_debounce_key_ttl() {
        // Setup: enqueue job with debounce
        // Act: check Redis TTL
        // Assert: TTL = delay + buffer
    }

    /// Тест: разные debounce_key независимы
    #[tokio::test]
    async fn test_different_keys_independent() {
        // Setup: job1 (key=A), job2 (key=B)
        // Act: enqueue both
        // Assert: both execute, no interference
    }
}
```

## Этап 3: Worker Configuration

### 3.1 Worker setup (`jobs/worker.rs`)

```rust
use apalis::prelude::*;
use apalis_redis::RedisStorage;
use apalis_cron::{CronStream, Schedule};
use std::str::FromStr;
use tower::ServiceBuilder;

use super::handlers::{handle_notification, handle_auto_archive, JobContext};
use super::middleware::DebounceLayer;
use super::types::{NotificationJob, AutoArchiveJob};

pub struct JobWorkers {
    monitor: Monitor,
}

impl JobWorkers {
    pub async fn new(
        redis_url: &str,
        ctx: JobContext,
        config: WorkerConfig,
    ) -> Result<Self, apalis::Error> {
        let redis_storage = RedisStorage::connect(redis_url).await?;

        // Notification worker с debounce
        let notification_storage: RedisStorage<NotificationJob> = redis_storage.clone();
        let notification_worker = WorkerBuilder::new("notifications")
            .layer(
                ServiceBuilder::new()
                    .layer(DebounceLayer::new(/* redis pool */))
                    .layer(RetryLayer::new(RetryPolicy::retries(3)))
                    .layer(TimeoutLayer::new(Duration::from_secs(30)))
            )
            .data(ctx.clone())
            .backend(notification_storage)
            .build_fn(handle_notification);

        // Auto-archive cron worker (раз в 5 минут)
        let archive_schedule = Schedule::from_str(&config.archive_cron)
            .unwrap_or_else(|_| Schedule::from_str("0 */5 * * * *").unwrap());
        let archive_worker = WorkerBuilder::new("auto-archive")
            .layer(TimeoutLayer::new(Duration::from_secs(300)))
            .data(ctx)
            .backend(CronStream::new(archive_schedule))
            .build_fn(handle_auto_archive);

        let monitor = Monitor::new()
            .register(notification_worker)
            .register(archive_worker);

        Ok(Self { monitor })
    }

    pub async fn run(self) -> Result<(), apalis::Error> {
        self.monitor.run().await
    }
}

#[derive(Clone)]
pub struct WorkerConfig {
    /// Задержка уведомлений в секундах
    pub notification_delay_secs: u64,
    /// Cron для автоархивирования
    pub archive_cron: String,
    /// Дней неактивности для архивации
    pub archive_after_days: u32,
    /// Concurrency для notification worker
    pub notification_concurrency: usize,
}

impl Default for WorkerConfig {
    fn default() -> Self {
        Self {
            notification_delay_secs: 30,
            archive_cron: "0 */5 * * * *".to_string(), // каждые 5 минут
            archive_after_days: 7,
            notification_concurrency: 4,
        }
    }
}
```

### 3.2 Интеграция в main.rs

```rust
// main.rs

mod jobs;

use jobs::{JobProducer, JobWorkers, JobContext, WorkerConfig};

#[tokio::main]
async fn main() {
    // ... existing setup ...

    // Initialize job system
    let job_ctx = JobContext {
        db: db.clone(),
        participants: state.participants.clone(),
        webhooks: state.webhooks.clone(),
    };

    let worker_config = WorkerConfig {
        notification_delay_secs: env::var("NOTIFICATION_DELAY_SECS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(30),
        archive_cron: env::var("ARCHIVE_CRON")
            .unwrap_or_else(|_| "0 */5 * * * *".to_string()),
        archive_after_days: env::var("ARCHIVE_AFTER_DAYS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(7),
        ..Default::default()
    };

    // Start workers in background
    let redis_url = env::var("REDIS_URL").expect("REDIS_URL required for job queue");
    let workers = JobWorkers::new(&redis_url, job_ctx, worker_config)
        .await
        .expect("Failed to initialize job workers");

    tokio::spawn(async move {
        if let Err(e) = workers.run().await {
            tracing::error!("Job workers error: {}", e);
        }
    });

    // ... start axum server ...
}
```

## Этап 4: Интеграция с send_message

### 4.1 Модификация send_message handler

```rust
// main.rs или отдельный модуль

async fn send_message(
    State(state): State<AppState>,
    UserId(sender_id): UserId,
    Path(dialog_id): Path<Uuid>,
    Json(req): Json<SendMessageRequest>,
) -> Result<Json<ApiResponse<MessageWithAttachments>>, ApiError> {
    // ... existing code: create message ...

    let message = state.messages.create(&message).await?;

    // ... existing code: attachments, increment unread ...

    // Schedule notifications for all participants except sender
    let participants = state.participants.list_by_dialog(dialog_id).await?;
    for participant in participants {
        if participant.user_id != sender_id {
            let job = NotificationJob::new(
                dialog_id,
                participant.user_id,
                message.id,
                sender_id,
            );

            // Debounced enqueue — если пользователь получит ещё сообщение
            // в течение delay, старая задача будет заменена
            if let Err(e) = state.jobs.enqueue_notification_debounced(job).await {
                tracing::warn!("Failed to enqueue notification: {}", e);
                // Не fail-им запрос — уведомления не критичны
            }
        }
    }

    // ... existing code: websocket broadcast, webhook ...
}
```

### 4.2 Добавление JobProducer в AppState

```rust
#[derive(Clone)]
struct AppState {
    // ... existing fields ...
    jobs: JobProducer,
}

impl AppState {
    fn new(
        db: PgPool,
        webhooks: WebhookSender,
        s3: S3Service,
        presence: PresenceService,
        jobs: JobProducer,
    ) -> Self {
        Self {
            // ... existing ...
            jobs,
        }
    }
}
```

## Этап 5: Тестирование

### 5.1 Unit тесты

| Модуль | Тесты |
|--------|-------|
| `jobs/types.rs` | Сериализация/десериализация, debounce_key генерация |
| `jobs/handlers.rs` | Логика проверки прочтения, notifications_enabled |
| `jobs/middleware/debounce.rs` | Debounce логика, TTL, независимость ключей |
| `jobs/producer.rs` | Enqueue с задержкой, debounce запись в Redis |

### 5.2 Integration тесты

```rust
// tests/jobs_integration.rs

#[tokio::test]
async fn test_notification_flow_end_to_end() {
    // 1. Создать диалог с двумя участниками
    // 2. Отправить сообщение от user1
    // 3. Проверить что job создан в Redis
    // 4. Подождать delay
    // 5. Проверить что webhook был отправлен для user2
}

#[tokio::test]
async fn test_notification_suppressed_when_read() {
    // 1. Создать диалог, отправить сообщение
    // 2. Mark as read до истечения delay
    // 3. Проверить что webhook НЕ был отправлен
}

#[tokio::test]
async fn test_debounce_multiple_messages() {
    // 1. Отправить 5 сообщений за 10 секунд
    // 2. Проверить что в Redis только 1 job (последний)
    // 3. Проверить что отправлен только 1 webhook
}

#[tokio::test]
async fn test_auto_archive_cron() {
    // 1. Создать диалог с last_message_at = 8 дней назад
    // 2. Запустить auto_archive job
    // 3. Проверить что все участники имеют is_archived = true
}
```

### 5.3 Mock для тестов

```rust
// tests/mocks/webhook_mock.rs

#[derive(Clone, Default)]
pub struct MockWebhookSender {
    calls: Arc<RwLock<Vec<WebhookEvent>>>,
}

impl MockWebhookSender {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn send(&self, event: WebhookEvent) {
        self.calls.write().await.push(event);
    }

    pub async fn call_count(&self) -> usize {
        self.calls.read().await.len()
    }

    pub async fn last_call(&self) -> Option<WebhookEvent> {
        self.calls.read().await.last().cloned()
    }
}
```

## Конфигурация

### Environment Variables

```env
# Job Queue
REDIS_URL=redis://localhost:6379

# Notifications
NOTIFICATION_DELAY_SECS=30        # Задержка перед проверкой прочтения
NOTIFICATION_ENABLED=true         # Включить/выключить систему уведомлений

# Auto-archive
ARCHIVE_CRON="0 */5 * * * *"      # Каждые 5 минут
ARCHIVE_AFTER_DAYS=7              # Архивировать после 7 дней неактивности
ARCHIVE_ENABLED=true              # Включить/выключить автоархивирование
```

## Чеклист реализации

### Этап 1: Базовая инфраструктура ✅
- [x] Добавить зависимости в Cargo.toml
- [x] Создать `jobs/mod.rs` с экспортами
- [x] Создать `jobs/types.rs` с NotificationJob, AutoArchiveJob
- [x] Создать `jobs/producer.rs` с JobProducer
- [x] Создать `jobs/handlers.rs` с handle_notification, handle_auto_archive
- [x] Тесты для types.rs

### Этап 2: Debounce ✅
- [x] Создать `jobs/middleware/mod.rs`
- [x] Создать `jobs/middleware/debounce.rs` с функциями `is_job_current`, `cleanup_debounce_key`
- [x] Debounce реализован в handler (проще чем tower Layer для apalis 0.6)

### Этап 3: Worker Configuration ✅
- [x] Создать `jobs/worker.rs` с WorkerConfig и start_workers
- [x] Интегрировать в main.rs
- [x] Тест конфигурации

### Этап 4: Интеграция ✅
- [x] Добавить JobProducer в AppState
- [x] Модифицировать send_message для enqueue notifications
- [x] Добавить notification.pending webhook event

### Этап 5: Production Ready
- [ ] Добавить метрики (jobs processed, debounced, failed)
- [ ] Load test с высокой нагрузкой
- [ ] Integration тесты с реальным Redis

## Риски и Mitigation

| Риск | Вероятность | Mitigation |
|------|-------------|------------|
| apalis API изменится (v0.6) | Средняя | Зафиксировать версию, следить за changelog |
| Redis недоступен | Низкая | Graceful degradation — уведомления пропускаются, логируем ошибку |
| Debounce race condition | Низкая | Использовать Redis SETNX или Lua script для атомарности |
| Высокая нагрузка на Redis | Низкая | Мониторинг, отдельный Redis instance если нужно |

## Ссылки

- [apalis documentation](https://docs.rs/apalis)
- [apalis GitHub](https://github.com/geofmureithi/apalis)
- [tower middleware](https://docs.rs/tower/latest/tower/)
- [План 002: Smart Notifications](./002-smart-notifications.md) — оригинальный план
