# Frontend Code Review: mtchat-vue

**Reviewer:** frontend-dev-2
**Date:** 2026-02-17
**Scope:** Full code review of `mtchat-vue/src/` (Vue 3 + TypeScript SDK)

---

## 1. Executive Summary

The mtchat-vue SDK is a well-structured, feature-rich embeddable chat component built on Vue 3 Composition API. The codebase demonstrates good architectural decisions: component registry pattern for UI abstraction, lightweight i18n without external dependencies, and a clean separation between SDK (API/WebSocket client) and UI layer.

**Overall Quality:** Good. The code is functional, well-typed, and follows most Vue 3 best practices. Below are specific findings organized by category.

**Severity Legend:**
- **[CRITICAL]** - Must fix before release (security, data loss, crashes)
- **[HIGH]** - Should fix before release (bugs, significant issues)
- **[MEDIUM]** - Recommended improvements (code quality, maintainability)
- **[LOW]** - Nice to have (style, minor improvements)

---

## 2. Security

### 2.1 [CRITICAL] XSS via v-html in ChatMessages.vue

**File:** `src/components/chat/ChatMessages.vue:704`, `src/components/chat/ChatMessages.vue:815`

```html
<div v-if="item.message.content" class="chat-messages__content" v-html="item.message.content"></div>
```

Message content is rendered with `v-html`. While the backend uses ammonia for HTML sanitization, the frontend has **no client-side sanitization** as a defense-in-depth measure. If a backend bug or API proxy bypass allows unsanitized HTML through, this is a direct XSS vector.

**Recommendation:** Add a client-side sanitization step (e.g., DOMPurify) before rendering with `v-html`, or at minimum validate that content only contains the expected allowed tags.

### 2.2 [HIGH] XSS via v-html in Icon.vue

**File:** `src/components/Icon.vue:128`

```html
<span class="mtchat-icon" v-html="svgContent" />
```

SVG content is rendered via `v-html`. While the icons are static imports bundled at build time (safe), the pattern is fragile. If the icon loading mechanism ever changes to accept dynamic/external SVGs, this becomes an XSS risk.

**Recommendation:** This is acceptable given the current static import design, but document the security constraint explicitly with a comment.

### 2.3 [MEDIUM] User ID in URL query parameters

**File:** `src/sdk/api.ts:57-66`

```typescript
private buildUrl(path: string, params: Record<string, string> = {}): string {
    const url = new URL(`${this.baseUrl}${path}`)
    url.searchParams.set('user_id', this.userId)
    ...
}
```

And in `src/sdk/client.ts:33`:

```typescript
url: `${wsUrl}?user_id=${config.userId}`,
```

User ID is passed as a query parameter in all requests, including the WebSocket URL. This means user IDs appear in server access logs, browser history, and potentially in referrer headers. There is no token-based authentication in the SDK - just a user ID.

**Recommendation:** The CLAUDE.md mentions "User Token" for Chat API, but the actual implementation uses bare `user_id` query params. Consider using a proper JWT/token in the `Authorization` header. The scope config is already base64-encoded and sent via a custom header, so this inconsistency should be resolved.

### 2.4 [MEDIUM] No CSRF protection

The API client uses `fetch` with custom headers but does not include any CSRF tokens. If the chat API is same-origin, it could be vulnerable to CSRF attacks.

**Recommendation:** If the API will be deployed same-origin, add CSRF token support. If it's always cross-origin (different domain), this is mitigated by CORS.

---

## 3. TypeScript & Type Safety

### 3.1 [HIGH] Pervasive `as any` type assertions in event handlers

**File:** `src/composables/useChat.ts:1322-1339`

```typescript
client.on('disconnected' as any, () => { ... })
client.on('presence.update', handlePresenceUpdate as any)
client.on('message.new', handleMessageNew as any)
client.on('message.read', handleMessageRead as any)
// ... all event handlers cast to `any`
```

Every WebSocket event handler registration uses `as any`. This defeats TypeScript's type checking for the most critical part of the application - the real-time event processing pipeline.

**Root cause:** The event handler types (`WsEventHandler`) expect `(event: WsEvent) => void`, but the actual handler functions have more specific parameter types. Also, `'disconnected'` is not in the `WsEventType` union.

**Recommendation:**
1. Add `'disconnected'` to `WsEventType`
2. Either make event handler types generic or use a discriminated union pattern:
   ```typescript
   client.on('message.new', (event: WsEvent) => {
     handleMessageNew(event.payload as MessageNewPayload)
   })
   ```
3. Define proper payload types for each event type

### 3.2 [MEDIUM] Inconsistent event payload handling

**File:** `src/composables/useChat.ts:958-975`

The `handleMessageNew` function accepts a union of two completely different shapes:

```typescript
function handleMessageNew(event: { payload?: { message?: Message }; id?: string; dialog_id?: string; ... }): void {
```

This suggests the WebSocket protocol is not stable and the frontend must handle multiple formats. This should be resolved at the protocol level rather than having defensive parsing in every handler.

### 3.3 [MEDIUM] Missing strict null checks in several places

**File:** `src/composables/useChat.ts:465`

```typescript
if (!oldestMessageId.value || new Date(oldestInResponse.sent_at) < new Date(oldestMessageId.value)) {
```

`oldestMessageId` is typed as `Ref<string | null>` and stores a message ID, but it's being compared as a Date, which would parse a UUID as an invalid date. This comparison only works because `new Date('uuid-string')` returns `Invalid Date` and the comparison happens to work due to NaN behavior.

### 3.4 [LOW] Redundant type annotations

Throughout `useChat.ts`, refs are declared with explicit type annotations that TypeScript can infer:

```typescript
const messages: Ref<Message[]> = ref([])
const isConnected: Ref<boolean> = ref(false)
```

While not wrong, this is unnecessarily verbose. Consider using `ref<Message[]>([])` for cleaner code.

---

## 4. Vue 3 Best Practices & Reactivity

### 4.1 [HIGH] Shared `isLoading` state across unrelated operations

**File:** `src/composables/useChat.ts`

A single `isLoading` ref is used for ALL async operations: loading dialogs, loading messages, joining, leaving, archiving, etc. This means:

1. Starting a dialog load can show a loading spinner in unrelated parts of the UI
2. Concurrent operations can clear each other's loading state via `finally` blocks
3. The `finally { isLoading.value = false }` pattern means the first operation to complete unsets loading for all

**Recommendation:** Split into operation-specific loading states: `isLoadingDialogs`, `isLoadingMessages`, `isJoining`, etc. Alternatively, use a counter-based approach.

### 4.2 [HIGH] Mutation-based reactivity in useFileUpload

**File:** `src/composables/useFileUpload.ts:99`

```typescript
pendingAttachments.value.push(pending)
```

And at lines 121-138, the `uploadFile` function directly mutates properties of items found via `.find()`:

```typescript
const item = pendingAttachments.value.find((a) => a.id === pending.id)
item.status = 'uploading'
item.progress = 0
...
item.s3Key = s3_key
item.status = 'uploaded'
```

This relies on Vue's deep reactivity tracking of the ref array. While this works with Vue 3's Proxy-based reactivity, it's inconsistent with the immutable update pattern used everywhere in `useChat.ts`. The pattern is fragile and can miss updates if the ref is replaced.

**Recommendation:** Use immutable updates for consistency:
```typescript
pendingAttachments.value = pendingAttachments.value.map(a =>
  a.id === pending.id ? { ...a, status: 'uploading', progress: 0 } : a
)
```

### 4.3 [MEDIUM] Duplicated helper functions

The following functions are duplicated across multiple files:

- `getInitials()` - in `ChatMessages.vue`, `ChatInfoPanel.vue`, `ReadersDialog.vue`
- `stripHtml()` - in `ChatMessages.vue`, `ChatInput.vue`
- `truncateText()` - in `ChatMessages.vue`, `ChatInput.vue`
- `getSenderDisplayName()` - in `ChatMessages.vue`, `ChatInput.vue`

**Recommendation:** Extract shared helpers to a `utils/` module and import them.

### 4.4 [MEDIUM] `useVirtualScroll` is always `true`

**File:** `src/components/chat/ChatMessages.vue:77`

```typescript
const useVirtualScroll = computed(() => true)
```

Yet the template still contains a full fallback `v-for` rendering path (lines 721-837) that is dead code. This adds ~120 lines of unreachable template code.

**Recommendation:** Remove the fallback rendering path if virtual scroll is always used. This simplifies the component and removes maintenance burden.

### 4.5 [MEDIUM] `console.log` debug statements left in production code

**File:** `src/composables/useChat.ts` - Multiple locations

Lines 1108-1113, 1117-1118, 1128, 1131, 1146-1150, 1155, 1166, 1169, 1186-1187, 1193-1194, 1200, and more contain `console.log` statements like:

```typescript
console.log('[MTChat] dialog.archived event:', { ... })
console.log('[MTChat] dialog.archived - found at index:', dialogIndex)
```

Also in `MTChat.vue:273-274`:
```typescript
console.log('[MTChat] jumpToMessage result:', { found, messageId })
console.log('[MTChat] calling scrollToMessage, ref exists:', !!messagesRef.value)
```

**Recommendation:** Remove all debug `console.log` statements or replace with a configurable debug logger that can be enabled/disabled.

### 4.6 [MEDIUM] Event handler memory leak potential

**File:** `src/composables/useChat.ts:1302-1339`

Event handlers are registered with `client.on()` in `onMounted` but never cleaned up in `onUnmounted`. The `onUnmounted` hook only calls `client.disconnect()` and `client.unsubscribe()`, but doesn't call `client.off()` for any of the registered handlers.

Since the client itself is created per-composable instance and `disconnect()` closes the WebSocket, this may not cause actual leaks in practice, but it's not clean. The `client.on()` returns an unsubscribe function that is never stored or called.

**Recommendation:** Store the cleanup functions and call them in `onUnmounted`:
```typescript
const cleanups: (() => void)[] = []
cleanups.push(client.on('message.new', handleMessageNew))
// ... in onUnmounted:
cleanups.forEach(fn => fn())
```

### 4.7 [LOW] Unnecessary `shallowRef` usage

**File:** `src/components/chat/ChatMessages.vue:63`

```typescript
const scrollerRef = shallowRef<any>(null)
```

Using `shallowRef` for a DOM/component ref is appropriate, but the type is `any`, losing all type safety. Consider using a more specific type or at least documenting why `any` is needed.

---

## 5. Architecture & Component Design

### 5.1 [HIGH] useChat composable is too large (1436 lines)

**File:** `src/composables/useChat.ts`

The composable handles: connection management, dialog CRUD, message CRUD, pagination (bidirectional), join/leave, archive/unarchive, pin/unpin, notifications, search, reply/edit state, online presence, WebSocket event handling, and more.

This violates the single responsibility principle and makes the composable difficult to test, maintain, and reason about.

**Recommendation:** Split into smaller composables:
- `useConnection` - WebSocket connection management
- `useDialogs` - dialog list operations (load, search, archive, pin)
- `useMessages` - message operations (load, send, edit, delete, pagination)
- `usePresence` - online status tracking
- `useChat` - orchestrating composable that combines the above

### 5.2 [MEDIUM] Missing error boundaries

No Vue error boundary components exist. If a child component (e.g., FileViewer, MessageEditor) throws an error, it will propagate up and potentially crash the entire chat widget.

**Recommendation:** Add `onErrorCaptured` in the root `MTChat.vue` component to gracefully handle child component errors and show a fallback UI.

### 5.3 [MEDIUM] No unit tests

There are zero tests for the frontend SDK. Key areas that need testing:
- `MTChatApi` - API client methods
- `useChat` - composable state management
- `useFileUpload` - file upload flow
- i18n - translation resolution and interpolation
- Type utilities - `getAttachmentType`, `isAllowedFileType`, `formatFileSize`

### 5.4 [LOW] Redundant `Composable Types` section header

**File:** `src/types/index.ts:541-542`

```typescript
// ============ Composable Types ============

// ============ Virtual Scroll Types ============
```

The "Composable Types" header appears twice (lines 541 and 567) with no content under the first one.

---

## 6. Performance

### 6.1 [MEDIUM] Expensive computed `allAttachments` iterates all messages

**File:** `src/components/MTChat.vue:123-131`

```typescript
const allAttachments = computed(() => {
  const attachments: Attachment[] = []
  for (const message of chat.messages.value) {
    if (message.attachments) {
      attachments.push(...message.attachments)
    }
  }
  return attachments
})
```

This iterates all loaded messages on every reactivity trigger to build a flat attachment list for the file viewer. For large message sets (1000+), this could be noticeable.

**Recommendation:** Compute lazily (only when FileViewer is opened) or cache the result with a watcher.

### 6.2 [MEDIUM] `getCurrentMessageReaders` called in template (not cached)

**File:** `src/components/MTChat.vue:306-321`, used at line 643:

```html
<ReadersDialog :readers="getCurrentMessageReaders()" ... />
```

This function is called directly in the template, meaning it runs on every render cycle. It contains nested loops over messages and participants.

**Recommendation:** Convert to a `computed` property based on `readersDialogMessage`.

### 6.3 [MEDIUM] `updateStickyDate` queries DOM on every scroll event

**File:** `src/components/chat/ChatMessages.vue:156-192`

The `updateStickyDate` function uses `querySelectorAll` on every scroll event to find messages and date dividers. This is a potential performance issue during rapid scrolling.

**Recommendation:** Debounce or throttle the sticky date update, or use `IntersectionObserver` instead of manual DOM queries.

### 6.4 [LOW] No debounce on window resize handler

**File:** `src/components/MTChat.vue:406-408`

```typescript
function handleWindowResize() {
  windowWidth.value = window.innerWidth
}
```

This triggers Vue reactivity on every resize event, which can cause layout thrashing.

**Recommendation:** Debounce the resize handler (e.g., 150ms).

---

## 7. Accessibility (a11y)

### 7.1 [HIGH] No ARIA attributes on interactive elements

Throughout the codebase, interactive elements lack ARIA attributes:

- **Chat sidebar dialog items:** No `role="listbox"` / `role="option"`, no `aria-selected`
- **Tab buttons:** No `role="tab"` / `role="tabpanel"` / `aria-selected`
- **Message actions:** No `aria-label` on icon-only buttons
- **Search input:** No `role="search"` on the container
- **File viewer overlay:** No `role="dialog"` / `aria-modal`

### 7.2 [MEDIUM] Missing keyboard navigation

- Dialog list items are `<div>` elements with `@click` but no keyboard support (Tab, Enter, arrow keys)
- Message action buttons appear on hover only - invisible to keyboard/screen reader users
- No focus management when dialogs (JoinDialog, ReadersDialog, FileViewer) open/close

### 7.3 [MEDIUM] Color-only status indicators

- Online status is indicated solely by a green dot (no text alternative)
- Connection status uses a colored dot (red/green) with text, which is acceptable
- Unread badge relies on color contrast

**Recommendation:** Add `aria-label` or `title` attributes to status indicators. Use patterns beyond color alone.

### 7.4 [LOW] Missing `lang` attribute consideration

The i18n system switches between ru/en/zh but doesn't set `lang` on the root element, which affects screen reader pronunciation.

---

## 8. Code Quality & Consistency

### 8.1 [MEDIUM] Inconsistent emit declaration styles

Some components use the new short-form:
```typescript
const emit = defineEmits<{
  connected: []
  'message-sent': [message: Message]
}>()
```

While others use the old call signature form:
```typescript
const emit = defineEmits<{
  (e: 'close'): void
  (e: 'join', data: JoinDialogRequest): void
}>()
```

**Recommendation:** Standardize on the new short-form syntax (available since Vue 3.3).

### 8.2 [MEDIUM] Scoped vs non-scoped styles inconsistency

Most components use `<style scoped>`, but some use `<style>` (non-scoped):
- `JoinDialog.vue` - `<style>` (non-scoped)
- `ReadersDialog.vue` - `<style>` (non-scoped)

Non-scoped styles leak into the host application and can cause conflicts.

**Recommendation:** Use `<style scoped>` consistently for all components in the library, or prefix all class names to avoid collisions.

### 8.3 [MEDIUM] Hardcoded color values in styles

Several components use hardcoded colors instead of CSS variables:

- `ChatInfoPanel.vue:289` - `#4CAF50` for online indicator (should be `--mtchat-success`)
- `ChatMessages.vue:1064` - `#22c55e` for online indicator
- `ChatHeader.vue:275` - `#ef4444` for disconnected status
- `MessageEditor.vue:927-928` - `rgba(59, 130, 246, 0.1)` for mention background
- `ChatInput.vue:248-249` - `#f59e0b` for edit indicator

The green values even differ between components (`#4CAF50` vs `#22c55e`).

**Recommendation:** Use the CSS variable system consistently (`--mtchat-success`, `--mtchat-danger`, `--mtchat-warning`).

### 8.4 [LOW] Token authentication not implemented

The SDK sends `user_id` as a query parameter and `X-Scope-Config` as a base64-encoded header, but there's no proper authentication token (JWT, API key) support despite the CLAUDE.md mentioning "User Token".

The `MTChatConfig` interface has no `token` field (despite the CLAUDE.md example showing `token: userToken`).

### 8.5 [LOW] Unused `vue-virtual-scroller` type declaration

**File:** `src/types/vue-virtual-scroller.d.ts` exists but I couldn't verify its contents. This should declare types for the `vue-virtual-scroller` package which doesn't ship its own.

---

## 9. Build & Package Configuration

### 9.1 [MEDIUM] `pdfjs-dist` and `vue-virtual-scroller` are in `dependencies`

**File:** `package.json:49-59`

These are bundled into the output since the library uses `vite-plugin-css-injected-by-js` and doesn't externalize them. This means consumers will get `pdfjs-dist` (large, ~2MB) included even if they never use the FileViewer component.

**Recommendation:** Consider:
1. Making FileViewer a lazy-loaded component
2. Making `pdfjs-dist` a peer dependency
3. Documenting the bundle size implications

### 9.2 [LOW] No `.eslintrc` or `prettier` config

The `package.json` has no ESLint or Prettier configuration, and no lint scripts.

**Recommendation:** Add ESLint with `@vue/eslint-config-typescript` and Prettier for consistent code formatting.

### 9.3 [LOW] `vue-virtual-scroller` uses beta version

**File:** `package.json:59`

```json
"vue-virtual-scroller": "^2.0.0-beta.8"
```

Using a beta dependency in production. While this is the only Vue 3 compatible version, it should be noted as a risk.

---

## 10. i18n Implementation

### 10.1 [MEDIUM] Duplicated code between `provideI18n` and `useI18n`

**File:** `src/i18n/useI18n.ts`

The `provideI18n` function (lines 19-98) and `useI18n` function (lines 104-197) contain completely duplicated implementations of `tt`, `formatDate`, and `formatDateDivider`. This is ~80 lines of duplicated logic.

**Recommendation:** Extract shared logic into a helper function and have both `provideI18n` and `useI18n` use it.

### 10.2 [LOW] No pluralization support

The i18n system uses simple `{count}` interpolation but has no pluralization rules. For example, `"{count} participants"` doesn't handle Russian pluralization (1 участник, 2 участника, 5 участников).

---

## 11. Component Registry Pattern

### 11.1 [LOW] Well-designed abstraction

The component registry pattern is well-executed:
- Clean `provide/inject` based architecture
- Default native HTML primitives that work without dependencies
- Properly typed interfaces for all registry components
- Easy to swap with PrimeVue or other frameworks

This is one of the strongest architectural decisions in the codebase.

---

## 12. Summary of Findings by Severity

| Severity | Count | Key Items |
|----------|-------|-----------|
| CRITICAL | 1 | XSS via v-html without client-side sanitization |
| HIGH | 6 | `as any` event handlers, shared isLoading, mutation-based reactivity, composable size, missing ARIA, auth token |
| MEDIUM | 17 | Various code quality, performance, accessibility, and consistency issues |
| LOW | 9 | Style nits, minor improvements |

---

## 13. Recommended Priority Actions

1. **Add client-side HTML sanitization** (DOMPurify) for message content rendered with `v-html`
2. **Remove all `console.log` debug statements** from production code
3. **Fix TypeScript event handler types** to eliminate `as any` casts
4. **Split `useChat` composable** into smaller, focused composables
5. **Standardize CSS variable usage** - remove hardcoded colors
6. **Add scoped styles** to JoinDialog and ReadersDialog
7. **Add ARIA attributes** to interactive elements
8. **Extract duplicated helpers** (`getInitials`, `stripHtml`, `truncateText`)
9. **Remove dead code** (fallback v-for rendering in ChatMessages)
10. **Add unit tests** for SDK, composables, and utility functions
