# PrimeVue Integration

The `@mtchat/vue-primevue` package provides seamless integration with [PrimeVue](https://primevue.org/) 4.x. It replaces MTChat's built-in UI primitives (buttons, dialogs, inputs, tabs, etc.) with their PrimeVue equivalents and maps your PrimeVue theme tokens for consistent styling.

## Installation

```bash
npm install @mtchat/vue-primevue
```

### Peer Dependencies

| Package | Version |
|---------|---------|
| `vue` | `^3.4.0` |
| `primevue` | `^4.0.0` |

!!! note
    You do **not** need to install `@mtchat/vue` separately. The PrimeVue package re-exports everything from the base SDK.

## Quick Start

Use `MTChatPrime` instead of `MTChat`:

```vue
<script setup lang="ts">
import { MTChatPrime, type MTChatConfig } from '@mtchat/vue-primevue'

const config: MTChatConfig = {
  baseUrl: 'https://chat.example.com',
  userId: currentUser.id,
  scopeConfig: {
    tenant_uid: currentUser.tenantId,
    scope_level1: currentUser.departments,
    scope_level2: currentUser.permissions,
  },
  userProfile: {
    displayName: currentUser.name,
    company: currentUser.company,
  },
  locale: 'en',
}
</script>

<template>
  <div style="height: 600px;">
    <MTChatPrime :config="config" theme="light" />
  </div>
</template>
```

`MTChatPrime` accepts the same props and events as `MTChat` (see [Full Mode](full-mode.md) and [Inline Mode](inline-mode.md)).

## How It Works

`MTChatPrime` is a thin wrapper that:

1. Provides a **PrimeVue component registry** via Vue's `provide`/`inject`
2. Applies a **theme CSS layer** that maps PrimeVue `--p-*` tokens to MTChat `--mtchat-*` variables
3. Re-exports all types, composables, and utilities from `@mtchat/vue`

### Component Mapping

| MTChat Primitive | PrimeVue Component |
|-----------------|-------------------|
| `MtButton` | `Button` |
| `MtDialog` | `Dialog` |
| `MtMenu` | `Menu` |
| `MtContextMenu` | `ContextMenu` |
| `MtInput` | `InputText` |
| `MtCheckbox` | `Checkbox` |
| `MtRadioButton` | `RadioButton` |
| `MtTabs` / `MtTab` | `Tabs` / `Tab` |
| `MtAccordion` / `MtAccordionPanel` | `Accordion` / `AccordionPanel` |

## Theme Integration

The PrimeVue integration automatically maps your PrimeVue preset tokens to MTChat variables. For example, if you use the Aura preset:

```
--p-primary-color    →  --mtchat-primary
--p-surface-0        →  --mtchat-bg
--p-text-color        →  --mtchat-text
--p-surface-border   →  --mtchat-border
```

### Custom Overrides

Override MTChat-specific variables on the `.mtchat-prime` wrapper:

```css
.mtchat-prime {
  --mtchat-primary: var(--p-primary-500);
  --mtchat-bg-hover: var(--p-surface-100);
}
```

## Advanced: Manual Registry Setup

If you need more control, use the base `MTChat` component with `provideRegistry`:

```vue
<script setup lang="ts">
import { MTChat, provideRegistry } from '@mtchat/vue'
import { primevueRegistry } from '@mtchat/vue-primevue'
import '@mtchat/vue/style.css'

// Register PrimeVue components as UI primitives
provideRegistry(primevueRegistry)
</script>

<template>
  <MTChat :config="config" />
</template>
```

This gives you full control over which registry components are used and how styles are applied.

### Custom Registry

Build a mixed registry with some PrimeVue components and some custom ones:

```typescript
import { provideRegistry, type PartialRegistry } from '@mtchat/vue'
import { PrimeButton, PrimeDialog } from '@mtchat/vue-primevue'
import MyCustomInput from './MyCustomInput.vue'

const customRegistry: PartialRegistry = {
  MtButton: PrimeButton,
  MtDialog: PrimeDialog,
  MtInput: MyCustomInput,  // Use your own input component
}

provideRegistry(customRegistry)
```

## Imports

All types, composables, and utilities are available from `@mtchat/vue-primevue`:

```typescript
// Component
import { MTChatPrime } from '@mtchat/vue-primevue'

// Types
import type {
  MTChatConfig,
  Message,
  DialogListItem,
  WsEvent,
} from '@mtchat/vue-primevue'

// Composables
import { useChat, useFileUpload } from '@mtchat/vue-primevue'

// SDK classes (advanced)
import { MTChatClient, MTChatApi } from '@mtchat/vue-primevue'

// Utilities
import {
  getAttachmentType,
  formatFileSize,
  ATTACHMENT_LIMITS,
} from '@mtchat/vue-primevue'
```
