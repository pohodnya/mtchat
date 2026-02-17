//! Webhook sender with HMAC signing and retry logic

use hmac::{Hmac, Mac};
use reqwest::Client;
use sha2::Sha256;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

use super::WebhookEvent;

type HmacSha256 = Hmac<Sha256>;

/// Webhook configuration
#[derive(Debug, Clone)]
pub struct WebhookConfig {
    /// Webhook endpoint URL
    pub url: String,
    /// Secret for HMAC-SHA256 signing
    pub secret: String,
    /// Maximum retry attempts (default: 3)
    pub max_retries: u32,
    /// Initial retry delay in milliseconds (default: 1000)
    pub retry_delay_ms: u64,
    /// Request timeout in seconds (default: 10)
    pub timeout_secs: u64,
}

impl WebhookConfig {
    pub fn new(url: impl Into<String>, secret: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            secret: secret.into(),
            max_retries: 3,
            retry_delay_ms: 1000,
            timeout_secs: 10,
        }
    }

    pub fn with_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    pub fn with_retry_delay(mut self, delay_ms: u64) -> Self {
        self.retry_delay_ms = delay_ms;
        self
    }

    pub fn with_timeout(mut self, timeout_secs: u64) -> Self {
        self.timeout_secs = timeout_secs;
        self
    }
}

impl Default for WebhookConfig {
    fn default() -> Self {
        Self {
            url: String::new(),
            secret: String::new(),
            max_retries: 3,
            retry_delay_ms: 1000,
            timeout_secs: 10,
        }
    }
}

/// Webhook sender service
///
/// Sends webhook events to configured endpoints in the background.
/// Uses a channel for async event delivery.
#[derive(Clone)]
pub struct WebhookSender {
    tx: mpsc::Sender<WebhookEvent>,
}

impl WebhookSender {
    /// Create a new webhook sender with the given configuration
    ///
    /// Returns the sender handle and spawns a background task for delivery.
    pub fn new(config: WebhookConfig) -> Self {
        let (tx, rx) = mpsc::channel::<WebhookEvent>(1000);

        // Spawn background worker
        tokio::spawn(webhook_worker(config, rx));

        Self { tx }
    }

    /// Create a no-op sender that discards all events
    ///
    /// Useful when webhooks are not configured.
    pub fn noop() -> Self {
        let (tx, mut rx) = mpsc::channel::<WebhookEvent>(1);

        // Spawn a task that just drains the channel
        tokio::spawn(async move {
            while rx.recv().await.is_some() {
                // Discard
            }
        });

        Self { tx }
    }

    /// Send a webhook event (non-blocking)
    ///
    /// Returns immediately. Event is delivered in background.
    pub async fn send(&self, event: WebhookEvent) {
        if let Err(e) = self.tx.send(event).await {
            error!("Failed to queue webhook event: {}", e);
        }
    }

    /// Check if the sender is still active
    pub fn is_active(&self) -> bool {
        !self.tx.is_closed()
    }
}

/// Background worker that processes webhook events
async fn webhook_worker(config: WebhookConfig, mut rx: mpsc::Receiver<WebhookEvent>) {
    let client = Client::builder()
        .timeout(Duration::from_secs(config.timeout_secs))
        .build()
        .expect("Failed to create HTTP client");

    info!("Webhook worker started, sending to: {}", config.url);

    while let Some(event) = rx.recv().await {
        let event_type = event.event_type.to_string();
        let event_id = event.id;

        match send_with_retry(&client, &config, &event).await {
            Ok(()) => {
                info!(
                    event_id = %event_id,
                    event_type = %event_type,
                    "Webhook delivered successfully"
                );
            }
            Err(e) => {
                error!(
                    event_id = %event_id,
                    event_type = %event_type,
                    error = %e,
                    "Webhook delivery failed after retries"
                );
            }
        }
    }

    info!("Webhook worker stopped");
}

/// Send event with retry logic
async fn send_with_retry(
    client: &Client,
    config: &WebhookConfig,
    event: &WebhookEvent,
) -> Result<(), String> {
    let payload =
        serde_json::to_string(event).map_err(|e| format!("Failed to serialize event: {}", e))?;

    let signature = compute_signature(&config.secret, &payload);

    let mut last_error = String::new();
    let mut delay = config.retry_delay_ms;

    for attempt in 0..=config.max_retries {
        if attempt > 0 {
            warn!(
                attempt = attempt,
                delay_ms = delay,
                "Retrying webhook delivery"
            );
            tokio::time::sleep(Duration::from_millis(delay)).await;
            delay *= 2; // Exponential backoff
        }

        match client
            .post(&config.url)
            .header("Content-Type", "application/json")
            .header("X-Webhook-Signature", &signature)
            .header("X-Webhook-Event", event.event_type.as_str())
            .header("X-Webhook-Id", event.id.to_string())
            .body(payload.clone())
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    return Ok(());
                }

                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                last_error = format!("HTTP {}: {}", status, body);

                // Don't retry 4xx errors (client errors)
                if status.is_client_error() {
                    return Err(last_error);
                }
            }
            Err(e) => {
                last_error = format!("Request failed: {}", e);
            }
        }
    }

    Err(last_error)
}

/// Compute HMAC-SHA256 signature
fn compute_signature(secret: &str, payload: &str) -> String {
    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC can take key of any size");
    mac.update(payload.as_bytes());
    let result = mac.finalize();
    format!("sha256={}", hex::encode(result.into_bytes()))
}

/// Verify HMAC-SHA256 signature
///
/// Use this on the receiving end to verify webhook authenticity.
#[allow(dead_code)]
pub fn verify_signature(secret: &str, payload: &str, signature: &str) -> bool {
    let expected = compute_signature(secret, payload);
    // Constant-time comparison to prevent timing attacks
    constant_time_eq(expected.as_bytes(), signature.as_bytes())
}

/// Constant-time comparison
#[allow(dead_code)]
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter()
        .zip(b.iter())
        .fold(0u8, |acc, (x, y)| acc | (x ^ y))
        == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_signature() {
        let secret = "test-secret";
        let payload = r#"{"type":"message.new"}"#;

        let sig = compute_signature(secret, payload);

        assert!(sig.starts_with("sha256="));
        assert_eq!(sig.len(), 7 + 64); // "sha256=" + 64 hex chars
    }

    #[test]
    fn test_verify_signature() {
        let secret = "my-webhook-secret";
        let payload = r#"{"id":"123","type":"message.new"}"#;

        let signature = compute_signature(secret, payload);

        assert!(verify_signature(secret, payload, &signature));
        assert!(!verify_signature(secret, payload, "sha256=invalid"));
        assert!(!verify_signature("wrong-secret", payload, &signature));
    }

    #[test]
    fn test_webhook_config_builder() {
        let config = WebhookConfig::new("https://example.com/webhook", "secret123")
            .with_retries(5)
            .with_retry_delay(2000)
            .with_timeout(30);

        assert_eq!(config.url, "https://example.com/webhook");
        assert_eq!(config.secret, "secret123");
        assert_eq!(config.max_retries, 5);
        assert_eq!(config.retry_delay_ms, 2000);
        assert_eq!(config.timeout_secs, 30);
    }
}
