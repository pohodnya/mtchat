# MTChat Development Makefile
# Локальная разработка без Docker

.PHONY: help install dev dev-backend dev-sdk dev-app db-up db-down migrate clean

# Цвета для вывода
CYAN := \033[36m
GREEN := \033[32m
YELLOW := \033[33m
RESET := \033[0m

help: ## Показать справку
	@echo "$(CYAN)MTChat Development Commands$(RESET)"
	@echo ""
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  $(GREEN)%-15s$(RESET) %s\n", $$1, $$2}'

# ============ Installation ============

install: ## Установить все зависимости
	@echo "$(CYAN)Installing backend dependencies...$(RESET)"
	cd mtchat-rust && cargo fetch
	@echo "$(CYAN)Installing SDK dependencies...$(RESET)"
	cd mtchat-vue && npm install
	@echo "$(CYAN)Installing example app dependencies...$(RESET)"
	cd mtchat-example && npm install
	@echo "$(GREEN)Done!$(RESET)"

install-tools: ## Установить dev tools (cargo-watch)
	cargo install cargo-watch

# ============ Database ============

minio-up: ## Запустить только PostgreSQL, Redis и MinIO (Docker)
	docker compose up -d minio minio-init

minio-down: ## Остановить PostgreSQL, Redis и MinIO
	docker compose stop minio

migrate: ## Применить миграции
	cd mtchat-rust && cargo sqlx migrate run

# ============ Development ============

dev-backend: ## Запустить backend с hot reload
	@echo "$(CYAN)Starting backend with hot reload...$(RESET)"
	@echo "$(YELLOW)Requires: cargo-watch (run 'make install-tools' if not installed)$(RESET)"
	cd mtchat-rust && cargo watch -x run

dev-sdk: ## Собирать SDK в watch mode
	@echo "$(CYAN)Building SDK in watch mode...$(RESET)"
	cd mtchat-vue && npm run dev

dev-app: ## Запустить example app (Vite dev server)
	@echo "$(CYAN)Starting example app...$(RESET)"
	cd mtchat-example && npm run dev

dev-sdk-build: ## Собрать SDK один раз
	@echo "$(CYAN)Building SDK...$(RESET)"
	cd mtchat-vue && npm run build

# ============ Combined Commands ============

dev: ## Запустить всё (в отдельных терминалах)
	@echo "$(CYAN)Starting development environment...$(RESET)"
	@echo ""
	@echo "$(YELLOW)Run these commands in separate terminals:$(RESET)"
	@echo ""
	@echo "  $(GREEN)Terminal 1 (DB):$(RESET)      make minio-up"
	@echo "  $(GREEN)Terminal 2 (Backend):$(RESET) make dev-backend"
	@echo "  $(GREEN)Terminal 3 (SDK):$(RESET)     make dev-sdk"
	@echo "  $(GREEN)Terminal 4 (App):$(RESET)     make dev-app"
	@echo ""
	@echo "$(YELLOW)Or use tmux/screen for a single terminal setup$(RESET)"

# ============ Build ============

build: ## Собрать всё для production
	@echo "$(CYAN)Building SDK...$(RESET)"
	cd mtchat-vue && npm run build
	@echo "$(CYAN)Building example app...$(RESET)"
	cd mtchat-example && npm run build
	@echo "$(CYAN)Building backend...$(RESET)"
	cd mtchat-rust && cargo build --release
	@echo "$(GREEN)Build complete!$(RESET)"

# ============ Testing ============

test: ## Запустить тесты
	@echo "$(CYAN)Running backend tests...$(RESET)"
	cd mtchat-rust && cargo test
	@echo "$(CYAN)Running SDK typecheck...$(RESET)"
	cd mtchat-vue && npm run typecheck

# ============ Cleanup ============

clean: ## Очистить build артефакты
	cd mtchat-rust && cargo clean
	cd mtchat-vue && rm -rf dist node_modules
	cd mtchat-example && rm -rf dist node_modules

# ============ Docker (full stack) ============

docker-up: ## Запустить всё в Docker
	docker compose up -d

docker-down: ## Остановить Docker
	docker compose down

docker-logs: ## Показать логи Docker
	docker compose logs -f

docker-rebuild: ## Пересобрать Docker контейнеры
	docker compose up -d --build
