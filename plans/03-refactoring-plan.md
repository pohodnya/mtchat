# Refactoring Plan: MTChat Open-Source Release

**Date:** 2026-02-17
**Based on:** 01-backend-review.md (backend-dev-1), 02-frontend-review.md (frontend-dev-1)

---

## Decision: What NOT to fix before release

The following issues are **acknowledged but deferred** — they require architectural changes that risk breaking existing functionality and are better addressed post-release:

1. **JWT Authentication (Backend CRITICAL)** — The current user_id-via-query-param design is by intent for the SDK architecture (host app manages auth, passes user_id). Adding JWT would change the integration contract. **Decision: Document as "host app responsibility" in security docs. Add JWT support as a post-release feature.**

2. **WebSocket subscription filtering** — Broadcasting all events to all connections works for current scale. Filtering is an optimization. **Defer to post-release.**

3. **Rate limiting** — Not critical for initial release. Document as recommended production configuration. **Defer.**

4. **CORS restriction** — SDK is designed to be embedded cross-origin. Document proper CORS configuration for production. **Defer.**

5. **CSRF protection** — Cross-origin API design mitigates this. **Defer.**

6. **Pluralization in i18n** — Nice to have. **Defer.**

7. **vue-virtual-scroller beta** — No alternative for Vue 3. **Accept and document.**

---

## Phase 1: CRITICAL & HIGH fixes (must do before release)

### Backend (backend-dev-1 + backend-dev-2)

| # | Issue | Severity | Owner | Est. |
|---|-------|----------|-------|------|
| B1 | Extract handlers from main.rs to src/api/ modules | HIGH | backend-dev-1 | L |
| B2 | Add lib.rs for testability | HIGH | backend-dev-1 | S |
| B3 | Fix admin token: constant-time comparison + read once at startup | HIGH | backend-dev-2 | S |
| B4 | Remove ~10 unused dependencies from Cargo.toml | MEDIUM | backend-dev-2 | S |
| B5 | Add database transactions for multi-step operations | MEDIUM | backend-dev-1 | M |
| B6 | Fix N+1 queries in list_dialogs (batch queries) | MEDIUM | backend-dev-2 | M |
| B7 | Remove unused config/default.toml or integrate config crate | LOW | backend-dev-2 | S |
| B8 | Standardize UUID versions (v7 everywhere) | LOW | backend-dev-2 | S |
| B9 | Dockerfile: multi-stage build with layer caching | LOW | backend-dev-2 | S |
| B10 | Fix WebSocket: remove random UUID fallback (return error instead) | MEDIUM | backend-dev-1 | S |

### Frontend (frontend-dev-1 + frontend-dev-2)

| # | Issue | Severity | Owner | Est. |
|---|-------|----------|-------|------|
| F1 | Add DOMPurify client-side sanitization for v-html | CRITICAL | frontend-dev-1 | S |
| F2 | Remove ALL console.log debug statements | MEDIUM | frontend-dev-1 | S |
| F3 | Fix TypeScript: eliminate all `as any` in event handlers | HIGH | frontend-dev-1 | M |
| F4 | Extract duplicated helpers (getInitials, stripHtml, truncateText, getSenderDisplayName) to utils/ | MEDIUM | frontend-dev-2 | S |
| F5 | Remove dead code: fallback v-for in ChatMessages (useVirtualScroll always true) | MEDIUM | frontend-dev-2 | S |
| F6 | Fix scoped styles: JoinDialog.vue, ReadersDialog.vue | MEDIUM | frontend-dev-2 | S |
| F7 | Standardize CSS variables: replace hardcoded colors | MEDIUM | frontend-dev-2 | M |
| F8 | Fix shared isLoading → split into operation-specific states | HIGH | frontend-dev-1 | M |
| F9 | Fix useFileUpload: use immutable updates instead of mutations | HIGH | frontend-dev-1 | S |
| F10 | Fix i18n duplication between provideI18n and useI18n | MEDIUM | frontend-dev-2 | S |
| F11 | Add ARIA attributes to interactive elements | HIGH | frontend-dev-2 | M |
| F12 | Standardize emit declaration styles (new short-form) | LOW | frontend-dev-2 | S |

**Size estimates:** S = Small (< 30 min), M = Medium (1-2 hours), L = Large (2-4 hours)

---

## Phase 2: Post-refactoring cleanup

These tasks can proceed after Phase 1 and in parallel:

1. **Add ESLint + Prettier config** for frontend (task #12 prerequisite)
2. **Add unit tests** for backend repositories and handlers (task #11)
3. **Add unit tests** for frontend SDK, composables, utilities (task #12)
4. **Lazy-load FileViewer** to avoid bundling pdfjs-dist for all consumers
5. **Add error boundary** (onErrorCaptured) in MTChat.vue
6. **Event handler cleanup** in onUnmounted (store cleanup functions)
7. **Debounce resize handler** and **throttle sticky date update**

---

## Phase 3: Release preparation (parallel track)

These tasks are independent and can start immediately:

- **Task #4:** Technical documentation (MkDocs)
- **Task #5:** User documentation & Quick Start
- **Task #6:** npm & DockerHub publishing plan
- **Task #7:** Docker Compose & Helm Chart
- **Task #8:** GitHub Actions CI/CD
- **Task #9:** Repository structure decision
- **Task #10:** README & GitHub formatting

---

## Work Distribution Summary

### backend-dev-1: B1, B2, B5, B10
- Primary: Extract main.rs into api/ modules (biggest task)
- Secondary: Add lib.rs, transactions, WebSocket fix

### backend-dev-2: B3, B4, B6, B7, B8, B9
- Primary: Admin auth fix, unused deps cleanup, N+1 fix
- Secondary: Config cleanup, UUID standardization, Dockerfile

### frontend-dev-1: F1, F2, F3, F8, F9
- Primary: DOMPurify, TypeScript fixes, loading states
- Secondary: Console.log cleanup, file upload reactivity

### frontend-dev-2: F4, F5, F6, F7, F10, F11, F12
- Primary: Helper extraction, dead code removal, CSS variables, ARIA
- Secondary: Scoped styles, i18n fix, emit standardization

---

## Principles

1. **Do not break existing functionality** — all changes must preserve current behavior
2. **One concern per commit** — make changes reviewable
3. **Test before and after** — verify the app works at each step
4. **No scope creep** — only fix what's listed above
