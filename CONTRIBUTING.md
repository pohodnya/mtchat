# Contributing to MTChat

Thank you for your interest in contributing to MTChat! This guide will help you get started.

## Development Setup

### Prerequisites

- [Rust](https://rustup.rs/) 1.75+
- [Node.js](https://nodejs.org/) 20+
- [Docker](https://docs.docker.com/get-docker/) and Docker Compose
- Git

### Getting Started

1. Fork and clone the repository:

```bash
git clone https://github.com/your-username/mtchat.git
cd mtchat
```

2. Start infrastructure services:

```bash
docker compose up -d postgres redis minio
```

3. Set up environment:

```bash
cp .env.example .env
# Edit .env with your local settings
```

4. Run the backend:

```bash
cd mtchat-rust
cargo run
```

5. Run the demo app (in another terminal):

```bash
cd mtchat-example
npm install
npm run dev
```

The demo app will be available at `http://localhost:5173`.

### Project Structure

| Directory | Description |
|-----------|-------------|
| `mtchat-rust/` | Backend API (Rust, axum) |
| `mtchat-vue/` | Vue.js SDK library |
| `mtchat-vue-primevue/` | PrimeVue integration wrapper |
| `mtchat-example/` | Demo application |
| `deploy/` | Docker Compose and Helm chart |

## Making Changes

### Branching

Create a feature branch from `master`:

```bash
git checkout -b feat/my-feature
```

### Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add message reactions
fix: resolve WebSocket reconnection on mobile
refactor: extract batch query helpers
docs: update Quick Start guide
ci: add arm64 Docker build
test: add integration tests for scope matching
```

### Code Style

**Rust:**
- Run `cargo fmt` before committing
- Run `cargo clippy -- -D warnings` to check for lint issues
- Add tests for new functionality

**TypeScript/Vue:**
- Run `npm run typecheck` to verify types
- Follow existing code patterns and naming conventions

### Testing

**Backend:**

```bash
cd mtchat-rust

# Unit tests
cargo test --lib

# Integration tests (requires PostgreSQL and Redis)
cargo test --tests
```

**Frontend:**

```bash
cd mtchat-vue
npm run typecheck
npm run build
```

## Pull Requests

1. Ensure your branch is up to date with `master`
2. All CI checks must pass (formatting, linting, tests, build)
3. Write a clear PR description explaining what changed and why
4. Link any related issues

### PR Checklist

- [ ] Code follows existing style conventions
- [ ] `cargo fmt` and `cargo clippy` pass
- [ ] New functionality has tests
- [ ] Documentation updated if applicable
- [ ] Commit messages follow Conventional Commits format

## Reporting Issues

Use [GitHub Issues](https://github.com/pohodnya/mtchat/issues) to report bugs or request features. Please include:

- Steps to reproduce (for bugs)
- Expected vs actual behavior
- Environment details (OS, browser, versions)

## Code of Conduct

Be respectful and constructive. We are committed to providing a welcoming and inclusive experience for everyone.

## License

By contributing, you agree that your contributions will be licensed under the [MIT License](LICENSE).
