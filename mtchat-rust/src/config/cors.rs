use std::time::Duration;
use tower_http::cors::{AllowHeaders, AllowMethods, AllowOrigin, CorsLayer};

pub struct CorsConfig {
    /// Comma-separated origins or "*" for all
    pub allowed_origins: String,
    /// Comma-separated methods (GET,POST,PUT,DELETE,OPTIONS)
    pub allowed_methods: String,
    /// Comma-separated headers or "*" for all
    pub allowed_headers: String,
    /// Allow credentials (cookies, auth headers)
    pub allow_credentials: bool,
    /// Max age for preflight cache (seconds)
    pub max_age: u64,
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            allowed_origins: "*".to_string(),
            allowed_methods: "GET,POST,PUT,DELETE,OPTIONS".to_string(),
            allowed_headers: "*".to_string(),
            allow_credentials: false,
            max_age: 3600,
        }
    }
}

impl CorsConfig {
    pub fn from_env() -> Self {
        Self {
            allowed_origins: std::env::var("CORS_ALLOWED_ORIGINS")
                .unwrap_or_else(|_| "*".to_string()),
            allowed_methods: std::env::var("CORS_ALLOWED_METHODS")
                .unwrap_or_else(|_| "GET,POST,PUT,DELETE,OPTIONS".to_string()),
            allowed_headers: std::env::var("CORS_ALLOWED_HEADERS")
                .unwrap_or_else(|_| "*".to_string()),
            allow_credentials: std::env::var("CORS_ALLOW_CREDENTIALS")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(false),
            max_age: std::env::var("CORS_MAX_AGE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(3600),
        }
    }

    pub fn into_layer(self) -> CorsLayer {
        let mut layer = CorsLayer::new();

        // Origins
        layer = if self.allowed_origins == "*" {
            layer.allow_origin(AllowOrigin::any())
        } else {
            let origins: Vec<_> = self
                .allowed_origins
                .split(',')
                .map(|s| s.trim().parse().unwrap())
                .collect();
            layer.allow_origin(origins)
        };

        // Methods
        layer = if self.allowed_methods == "*" {
            layer.allow_methods(AllowMethods::any())
        } else {
            let methods: Vec<_> = self
                .allowed_methods
                .split(',')
                .filter_map(|s| s.trim().parse().ok())
                .collect();
            layer.allow_methods(methods)
        };

        // Headers
        layer = if self.allowed_headers == "*" {
            layer.allow_headers(AllowHeaders::any())
        } else {
            let headers: Vec<_> = self
                .allowed_headers
                .split(',')
                .filter_map(|s| s.trim().parse().ok())
                .collect();
            layer.allow_headers(headers)
        };

        // Credentials & Max Age
        if self.allow_credentials {
            layer = layer.allow_credentials(true);
        }
        layer = layer.max_age(Duration::from_secs(self.max_age));

        layer
    }
}
