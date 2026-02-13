# Plan 017: Virtual Scroll for Chat Messages

**Status:** Implemented
**Date:** 2026-02-13

## Summary

Implement virtual scrolling in ChatMessages using `vue-virtual-scroller` (DynamicScroller) to support 1000+ messages without performance degradation. The library is an optional peer dependency with automatic fallback to standard v-for rendering.

## Problem

When a chat has hundreds or thousands of messages, rendering all DOM nodes causes:
- Slow initial render
- Laggy scrolling
- High memory usage
- Poor performance on mobile devices

## Solution

Use vue-virtual-scroller's DynamicScroller component to render only visible items plus a small buffer. This keeps DOM node count constant (~20-30) regardless of total message count.

## Key Changes

### 1. package.json (mtchat-vue)

Added vue-virtual-scroller as optional peer dependency:

```json
"peerDependencies": {
  "vue": "^3.4.0",
  "vue-virtual-scroller": "^2.0.0-beta.8"
},
"peerDependenciesMeta": {
  "vue-virtual-scroller": {
    "optional": true
  }
}
```

### 2. types/index.ts

Added VirtualItem type for flat virtual list:

```typescript
export type VirtualItemType = 'message' | 'date-divider' | 'unread-divider'

export interface VirtualItem {
  id: string
  type: VirtualItemType
  message?: Message
  date?: string
  dateKey?: string
}
```

### 3. types/vue-virtual-scroller.d.ts

Created type declarations for vue-virtual-scroller (optional dependency).

### 4. ChatMessages.vue

Major changes:
- Dynamic import of vue-virtual-scroller (non-blocking)
- `virtualItems` computed property transforms messages into flat list with dividers
- Conditional rendering: DynamicScroller when available and messages >= 100, fallback v-for otherwise
- Updated scroll handling for virtual scroller
- Scroll position preservation on prepend (infinite scroll)
- Sticky date header positioned outside scroller

Key implementation details:

```typescript
// Threshold for using virtual scroll
const VIRTUAL_SCROLL_THRESHOLD = 100

// Dynamic import (non-blocking)
const DynamicScroller = shallowRef<any>(null)
const DynamicScrollerItem = shallowRef<any>(null)

import('vue-virtual-scroller')
  .then((vvs) => {
    DynamicScroller.value = vvs.DynamicScroller
    DynamicScrollerItem.value = vvs.DynamicScrollerItem
  })
  .catch(() => {
    // Fallback to v-for
  })

// Use virtual scroll when available and message count exceeds threshold
const useVirtualScroll = computed(() => {
  return DynamicScroller.value !== null && props.messages.length >= VIRTUAL_SCROLL_THRESHOLD
})
```

### 5. mtchat-example/package.json

Added vue-virtual-scroller dependency for demo app.

### 6. mtchat-example/src/main.ts

Import vue-virtual-scroller CSS:

```typescript
import 'vue-virtual-scroller/dist/vue-virtual-scroller.css'
```

## Features

| Feature | Implementation |
|---------|---------------|
| Variable item heights | DynamicScroller with size-dependencies prop |
| Sticky date headers | Floating overlay outside scroller |
| Scroll to message | scrollToItem(index) + highlight |
| Prepend messages | Save scrollTop/scrollHeight before, restore delta after |
| Unread divider | Injected as VirtualItem with type 'unread-divider' |
| Fallback mode | Standard v-for when < 100 messages or library not installed |

## Verification

1. Install vue-virtual-scroller in consuming app: `npm install vue-virtual-scroller`
2. Import CSS: `import 'vue-virtual-scroller/dist/vue-virtual-scroller.css'`
3. Load 500+ messages in a dialog
4. Test infinite scroll (load older messages when scrolling up)
5. Test scroll-to-message (click reply quote)
6. Test sticky date headers
7. Check DOM inspector - should see ~20-30 DOM nodes regardless of message count
8. Test without vue-virtual-scroller installed - should fallback to v-for

## Performance

- With 1000 messages: ~25 DOM nodes instead of ~1000
- Smooth 60fps scrolling
- Memory usage stays constant
- Fast initial render
