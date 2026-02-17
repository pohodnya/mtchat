# Stage 1: Install cargo-chef on pinned Rust image
FROM rust:1.85-bookworm AS chef
RUN cargo install cargo-chef
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
RUN cargo build --release

# Stage 5: Runtime (minimal image)
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

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
