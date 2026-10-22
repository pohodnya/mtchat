# Stage 1: Base image with cargo-chef preinstalled
FROM lukemathwalker/cargo-chef:latest-rust-1.93-bookworm AS chef
WORKDIR /app

# Stage 2: Analyze dependencies
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Stage 3: Build dependencies (cached unless Cargo.toml/Cargo.lock change)
FROM chef AS builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Stage 4: Build application (only this layer rebuilds on source changes)
COPY . .
RUN cargo build --release --bin multitenancy-chat-api

# Stage 5: Runtime (minimal image)
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Distrust the compromised eMudhra (emSign) roots until Debian ships a fixed
# ca-certificates (Alpine equivalent: ca-certificates>=20261020-r0)
RUN sed -i -E 's|^(mozilla/emSign_.*\.crt)$|!\1|' /etc/ca-certificates.conf \
    && update-ca-certificates --fresh \
    && ! grep -q '^mozilla/emSign_' /etc/ca-certificates.conf

WORKDIR /app

COPY --from=builder /app/target/release/multitenancy-chat-api .
COPY --from=builder /app/migrations ./migrations

RUN useradd -m -u 1000 appuser && chown -R appuser:appuser /app
USER appuser

ENV RUST_LOG=multitenancy_chat_api=info,tower_http=info

EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=10s --start-period=60s --retries=5 \
    CMD curl -f http://localhost:8080/health || exit 1

CMD ["./multitenancy-chat-api"]
