<script setup lang="ts">
/**
 * PrimeDialog - PrimeVue Dialog adapter
 *
 * Uses appendTo="self" to keep the dialog inside the component tree,
 * so PrimeVue's scoped preset styles and CSS layers apply correctly.
 * CSS below maps PrimeVue tokens to MTChat CSS variables for the dialog.
 */

import { computed } from 'vue'
import Dialog from 'primevue/dialog'
import type { MtDialogProps, MtDialogEmits } from '@mtchat/vue'

const props = withDefaults(defineProps<MtDialogProps>(), {
  modal: true,
  closable: true,
  maxWidth: '400px',
  draggable: false,
})

const emit = defineEmits<MtDialogEmits>()

const dialogPt = computed(() => ({
  root: {
    class: props.theme ? `mtchat-dialog--${props.theme}` : null,
  },
}))

function handleVisibleChange(value: boolean) {
  emit('update:visible', value)
  if (!value) {
    emit('close')
  }
}
</script>

<template>
  <Dialog
    :visible="visible"
    :header="header"
    :modal="modal"
    :closable="closable"
    :draggable="draggable"
    :style="{ maxWidth, width: '100%' }"
    :pt="dialogPt"
    append-to="self"
    @update:visible="handleVisibleChange"
  >
    <slot />
    <template v-if="$slots.footer" #footer>
      <slot name="footer" />
    </template>
  </Dialog>
</template>

<style>
/*
 * MTChat CSS variables mapping from PrimeVue tokens.
 * Used by JoinDialog content (labels, inputs, etc.)
 */
.mtchat-dialog--light {
  --mtchat-bg: var(--p-surface-0, #ffffff);
  --mtchat-text: var(--p-text-color, #3f3f46);
  --mtchat-text-secondary: var(--p-text-muted-color, #71717a);
  --mtchat-border: var(--p-surface-200, #e4e4e7);
  --mtchat-bg-hover: var(--p-surface-100, #f4f4f5);
  --mtchat-hover: var(--p-surface-100, #f4f4f5);
  --mtchat-primary: var(--p-primary-color, #6366f1);
  --mtchat-input-bg: var(--p-surface-0, #ffffff);
  --mtchat-input-border: var(--p-surface-300, #d4d4d8);
}

.mtchat-dialog--dark {
  --mtchat-bg: var(--p-surface-900, #18181b);
  --mtchat-text: var(--p-surface-0, #fafafa);
  --mtchat-text-secondary: var(--p-surface-400, #a1a1aa);
  --mtchat-border: var(--p-surface-700, #3f3f46);
  --mtchat-bg-hover: var(--p-surface-700, #3f3f46);
  --mtchat-hover: var(--p-surface-700, #3f3f46);
  --mtchat-primary: var(--p-primary-400, #818cf8);
  --mtchat-input-bg: var(--p-surface-900, #18181b);
  --mtchat-input-border: var(--p-surface-700, #3f3f46);
}
</style>
