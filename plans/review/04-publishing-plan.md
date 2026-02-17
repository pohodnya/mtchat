# Publishing Plan: npm & DockerHub

## Overview

This document describes the release process for all MTChat artifacts:
- **@mtchat/vue** -- core Vue.js SDK (npm)
- **@mtchat/vue-primevue** -- PrimeVue integration wrapper (npm)
- **mtchat/backend** -- Rust API server (DockerHub + GHCR)
- **Helm Chart** -- Kubernetes deployment (GitHub Releases)

All releases are triggered by a single git tag and automated via GitHub Actions.

---

## 1. Versioning Strategy

### Unified Semver

All artifacts share the **same version number** derived from the git tag.

```
Tag: v0.4.0
  @mtchat/vue       → 0.4.0
  @mtchat/vue-primevue → 0.4.0
  mtchat/backend     → 0.4.0
  Helm chart         → 0.4.0 (appVersion)
```

### Semver Rules

| Change | Version Bump | Example |
|--------|-------------|---------|
| Breaking API/protocol changes | MAJOR | v1.0.0 → v2.0.0 |
| New features, backward-compatible | MINOR | v0.4.0 → v0.5.0 |
| Bug fixes, patches | PATCH | v0.4.0 → v0.4.1 |
| Pre-release / RC | Pre-release suffix | v1.0.0-rc.1 |

### Pre-1.0 Convention

While in `0.x.y`:
- MINOR bumps may include breaking changes (semver allows this for 0.x)
- Document breaking changes clearly in CHANGELOG
- Target v1.0.0 for the first stable public release

### Version Sources

| Artifact | Version Source | Updated By |
|----------|---------------|------------|
| @mtchat/vue | `package.json` version field | CI: `npm version` from tag |
| @mtchat/vue-primevue | `package.json` version field | CI: `npm version` from tag |
| mtchat/backend | Docker image tags | CI: docker/metadata-action from tag |
| Helm chart | `Chart.yaml` version + appVersion | Manual (pre-release step) |
| Cargo.toml | `version` field | Manual (pre-release step) |

---

## 2. npm Publishing

### 2.1 Package: @mtchat/vue

**Registry:** https://www.npmjs.com/package/@mtchat/vue

**Package contents** (controlled by `files` in package.json):
```
dist/
  mtchat-vue.js        # ESM bundle
  mtchat-vue.umd.cjs   # UMD bundle (CommonJS)
  style.css             # Extracted CSS
  index.d.ts            # TypeScript declarations
```

**Peer dependencies:**
- `vue: ^3.4.0`

**Runtime dependencies** (bundled):
- @tiptap/* (rich text editor)
- dompurify (HTML sanitization)
- pdfjs-dist (PDF viewer)
- vue-virtual-scroller

### 2.2 Package: @mtchat/vue-primevue

**Registry:** https://www.npmjs.com/package/@mtchat/vue-primevue

**Package contents:**
```
dist/
  mtchat-vue-primevue.js
  mtchat-vue-primevue.umd.cjs
  theme/aura.css
  index.d.ts
```

**Peer dependencies:**
- `vue: ^3.4.0`
- `primevue: ^4.0.0`

**Build dependency:** Requires @mtchat/vue to be built first (file: reference in dev).

**CI publish order:** `npm-vue` job completes before `npm-primevue` starts.

### 2.3 npm Scope & Access

- **Scope:** `@mtchat` (organization scope on npm)
- **Access:** `--access public` (open-source)
- **Token:** `NPM_TOKEN` secret in GitHub (automation token with publish rights to @mtchat scope)

### 2.4 Pre-publish Checklist (npm)

Before first publish:
- [ ] Create `@mtchat` organization on npmjs.com
- [ ] Generate automation token: npmjs.com > Access Tokens > Granular Access Token
- [ ] Grant token publish access to @mtchat scope
- [ ] Add `NPM_TOKEN` to GitHub repository secrets
- [ ] Update `repository.url` in both package.json files to actual GitHub URL
- [ ] Verify `files` field includes only `dist/` (no source code leaked)
- [ ] Verify `.npmignore` or `files` field excludes: `src/`, `node_modules/`, `*.config.*`
- [ ] Run `npm pack --dry-run` locally to verify package contents

### 2.5 Version Injection in CI

The release workflow sets the version from the git tag at build time:

```yaml
# In release.yml:
- name: Set version from tag
  run: npm version "${GITHUB_REF_NAME#v}" --no-git-tag-version
```

This means package.json in the repo can stay at `1.0.0` (or any placeholder) -- the actual published version always comes from the tag.

---

## 3. Docker Publishing

### 3.1 Image: mtchat/backend

**Registries:**
- DockerHub: `mtchat/backend`
- GHCR: `ghcr.io/<org>/mtchat/backend`

**Architectures:** `linux/amd64`, `linux/arm64`

**Tag strategy** (handled by docker/metadata-action):

| Tag Format | Example | When |
|-----------|---------|------|
| `{version}` | `0.4.0` | Every release |
| `{major}.{minor}` | `0.4` | Every release |
| `{major}` | `1` | Only for v1.0.0+ (not v0.x) |
| `latest` | `latest` | Every release |

For v0.4.1, the following tags are pushed:
```
mtchat/backend:0.4.1
mtchat/backend:0.4
mtchat/backend:latest
ghcr.io/<org>/mtchat/backend:0.4.1
ghcr.io/<org>/mtchat/backend:0.4
ghcr.io/<org>/mtchat/backend:latest
```

### 3.2 Dockerfile

Location: `mtchat-rust/docker/Dockerfile`

Multi-stage build with cargo-chef:
1. **chef** -- installs cargo-chef on pinned `rust:1.82-bookworm`
2. **planner** -- analyzes dependency graph
3. **builder** -- builds dependencies (cached layer), then application
4. **runtime** -- `debian:bookworm-slim` with binary + migrations

### 3.3 Pre-publish Checklist (Docker)

- [ ] Create `mtchat` organization on DockerHub
- [ ] Create `backend` repository under mtchat org
- [ ] Generate access token: DockerHub > Account Settings > Security > New Access Token
- [ ] Add `DOCKERHUB_USERNAME` and `DOCKERHUB_TOKEN` to GitHub secrets
- [ ] GHCR is automatic (uses `GITHUB_TOKEN`, no extra setup)
- [ ] Write DockerHub repository description (short + full from README)
- [ ] Add DockerHub repository links (GitHub, documentation)

---

## 4. Helm Chart Publishing

### 4.1 Distribution

The Helm chart is distributed via **GitHub Releases** as an OCI artifact or tarball attachment.

Location: `deploy/helm/mtchat/`

### 4.2 Version Sync

`Chart.yaml` must be updated manually before tagging:
```yaml
version: 0.4.0      # Chart version (matches release)
appVersion: "0.4.0"  # Docker image version
```

### 4.3 Installation

```bash
# From GitHub Release tarball
helm install mtchat ./mtchat-0.4.0.tgz

# From local clone
helm install mtchat deploy/helm/mtchat/
```

### 4.4 Future: OCI Registry

When the project matures, consider publishing to an OCI-compatible Helm registry:
```bash
helm push mtchat-0.4.0.tgz oci://ghcr.io/<org>/charts
```

This requires adding a step to the release workflow.

---

## 5. CHANGELOG Generation

### 5.1 Commit Convention

Adopt **Conventional Commits** for meaningful auto-generated changelogs:

```
feat: add message reactions support
fix: resolve WebSocket reconnection on mobile
refactor: extract batch query helpers
docs: update Quick Start guide
ci: add arm64 Docker build
chore: bump dependencies
```

### 5.2 Types Mapping

| Prefix | CHANGELOG Section | Bumps |
|--------|------------------|-------|
| `feat:`, `add:`, `new:` | Features | MINOR |
| `fix:`, `bug:`, `patch:` | Bug Fixes | PATCH |
| `refactor:`, `perf:` | Other Changes | -- |
| `docs:`, `ci:`, `chore:`, `style:`, `test:` | Other Changes | -- |
| `BREAKING CHANGE:` in body | Breaking Changes | MAJOR |

### 5.3 Auto-generation Process

The release workflow generates the changelog automatically:

1. Finds the previous tag (`git tag --sort=-version:refname`)
2. Collects commits between previous tag and current tag
3. Categorizes by prefix (feat/fix/other)
4. Formats as markdown sections
5. Appends package installation commands
6. Attaches to GitHub Release

### 5.4 Manual CHANGELOG.md

In addition to auto-generated release notes, maintain a `CHANGELOG.md` at the repo root for:
- More detailed descriptions of features
- Migration guides for breaking changes
- Links to relevant issues/PRs

Format:
```markdown
# Changelog

## [0.5.0] - 2026-02-20

### Features
- Message reactions with emoji picker (#42)
- Typing indicators in real-time (#45)

### Bug Fixes
- Fix WebSocket reconnection on iOS Safari (#41)

### Breaking Changes
- Renamed `MTChat` prop `scopeConfig` to `scope` (#43)
  - Migration: replace `scopeConfig` with `scope` in your config object
```

---

## 6. Release Process

### 6.1 Standard Release Flow

```
1. Code freeze on master
2. Update versions (manual)
3. Create & push tag
4. CI runs automatically
5. Verify artifacts
```

### 6.2 Step-by-step

#### Step 1: Prepare

Ensure all changes are merged to `master` and CI is green.

```bash
git checkout master
git pull origin master
```

#### Step 2: Update Versions

Update version numbers in files that aren't auto-set by CI:

```bash
# Cargo.toml (informational, not published to crates.io)
# Edit mtchat-rust/Cargo.toml: version = "0.4.0"

# Helm chart
# Edit deploy/helm/mtchat/Chart.yaml: version and appVersion

# CHANGELOG.md
# Add new section with changes since last release
```

#### Step 3: Commit Version Bump

```bash
git add -A
git commit -m "chore: bump version to 0.4.0"
git push origin master
```

#### Step 4: Create Tag

```bash
git tag -a v0.4.0 -m "Release v0.4.0"
git push origin v0.4.0
```

#### Step 5: Monitor CI

The `release.yml` workflow triggers automatically:
1. **validate** -- runs all tests (Rust + Vue)
2. **docker** -- builds multi-arch image, pushes to DockerHub + GHCR
3. **npm-vue** -- publishes @mtchat/vue
4. **npm-primevue** -- publishes @mtchat/vue-primevue (after npm-vue)
5. **github-release** -- creates GitHub Release with changelog

Monitor at: `https://github.com/<org>/mtchat/actions`

#### Step 6: Verify

```bash
# Check npm
npm view @mtchat/vue versions
npm view @mtchat/vue-primevue versions

# Check Docker
docker pull mtchat/backend:0.4.0
docker run --rm mtchat/backend:0.4.0 --version  # if supported

# Check GitHub Release
gh release view v0.4.0
```

### 6.3 Pre-release / RC

For release candidates:

```bash
git tag -a v1.0.0-rc.1 -m "Release v1.0.0-rc.1"
git push origin v1.0.0-rc.1
```

This triggers the same workflow but:
- GitHub Release is marked as **prerelease** (`contains(tag, '-')`)
- npm gets a pre-release version (`1.0.0-rc.1`)
- Docker gets tag `1.0.0-rc.1` (no `latest` tag update needed -- metadata-action handles it)

### 6.4 Hotfix Release

```bash
git checkout master
# Apply fix
git commit -m "fix: critical auth bypass in scope matching"
git push origin master

# Bump patch
# Edit Cargo.toml, Chart.yaml
git commit -m "chore: bump version to 0.4.1"
git tag -a v0.4.1 -m "Release v0.4.1 - hotfix"
git push origin v0.4.1
```

---

## 7. Release Checklist

### First-time Setup (one-time)

- [ ] **npm:** Create `@mtchat` organization on npmjs.com
- [ ] **npm:** Generate automation token for @mtchat scope
- [ ] **npm:** Add `NPM_TOKEN` to GitHub repository secrets
- [ ] **Docker:** Create `mtchat` organization on DockerHub
- [ ] **Docker:** Create `mtchat/backend` repository
- [ ] **Docker:** Add `DOCKERHUB_USERNAME` and `DOCKERHUB_TOKEN` to GitHub secrets
- [ ] **GitHub:** Ensure `GITHUB_TOKEN` has `contents: write` and `packages: write` permissions
- [ ] **npm:** Update `repository.url` in package.json files to real GitHub URL
- [ ] **Helm:** Update `home` in Chart.yaml to real GitHub URL
- [ ] **Test:** Do a dry run with `v0.0.1-test.1` tag to verify the full pipeline

### Per-release Checklist

```markdown
## Release v{VERSION}

### Pre-release
- [ ] All PRs merged, master is green
- [ ] Update `mtchat-rust/Cargo.toml` version
- [ ] Update `deploy/helm/mtchat/Chart.yaml` version + appVersion
- [ ] Update `CHANGELOG.md` with new section
- [ ] Commit: `chore: bump version to {VERSION}`
- [ ] Push to master

### Release
- [ ] Create annotated tag: `git tag -a v{VERSION} -m "Release v{VERSION}"`
- [ ] Push tag: `git push origin v{VERSION}`
- [ ] Monitor CI workflow: Actions tab

### Post-release Verification
- [ ] GitHub Release created with changelog
- [ ] `npm view @mtchat/vue@{VERSION}` returns package info
- [ ] `npm view @mtchat/vue-primevue@{VERSION}` returns package info
- [ ] `docker pull mtchat/backend:{VERSION}` succeeds
- [ ] `docker pull ghcr.io/<org>/mtchat/backend:{VERSION}` succeeds
- [ ] Update DockerHub description if needed
- [ ] Announce release (if applicable)
```

---

## 8. Rollback Procedures

### npm Rollback

npm packages are **immutable** -- you cannot overwrite a published version. Options:

1. **Deprecate:** `npm deprecate @mtchat/vue@0.4.0 "Critical bug, use 0.4.1"`
2. **Unpublish** (within 72h only): `npm unpublish @mtchat/vue@0.4.0`
3. **Patch release:** publish `0.4.1` with the fix (preferred)

### Docker Rollback

Docker tags are **mutable** -- you can retag:

```bash
# Point latest to previous version
docker buildx imagetools create \
  --tag mtchat/backend:latest \
  mtchat/backend:0.3.0
```

Or users can pin: `mtchat/backend:0.3.0` in their docker-compose.yml.

### GitHub Release Rollback

```bash
# Delete the release (keeps tag)
gh release delete v0.4.0

# Or delete tag too
git tag -d v0.4.0
git push origin :refs/tags/v0.4.0
```

---

## 9. Security Considerations

- **npm token:** Use granular access token scoped to `@mtchat` packages only
- **DockerHub token:** Use access token (not password), scope to read/write only
- **GitHub secrets:** Never log secrets in CI; use `${{ secrets.* }}` syntax only
- **Supply chain:** Enable npm 2FA for the @mtchat organization
- **Docker signing:** Consider enabling Docker Content Trust (DCT) for image signing
- **Provenance:** GitHub Actions generates SLSA provenance for Docker images automatically with buildx v0.10+

---

## 10. Future Improvements

| Improvement | Priority | Notes |
|------------|----------|-------|
| Automated version bumping | Medium | Use `standard-version` or `release-please` |
| OCI Helm chart registry | Low | Publish to ghcr.io for `helm pull oci://` |
| Crates.io publishing | Low | Only if backend is used as a Rust library |
| npm provenance | Medium | `npm publish --provenance` for SLSA attestation |
| Release notes template | Low | GitHub issue/PR template for release notes |
| Automated security scanning | Medium | Trivy for Docker images, npm audit in CI |
