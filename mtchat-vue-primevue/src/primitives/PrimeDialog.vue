<script setup lang="ts">
/**
 * PrimeDialog - PrimeVue Dialog adapter
 */

import { computed } from 'vue'
import Dialog from 'primevue/dialog'
import type { MtDialogProps, MtDialogEmits } from '@mtchat/vue'

const props = withDefaults(defineProps<MtDialogProps>(), {
  modal: true,
  closable: true,
  maxWidth: '400px',
  draggable: false,
  theme: 'light',
})

const emit = defineEmits<MtDialogEmits>()

const dialogPt = computed(() => ({
  root: {
    class: `mtchat-dialog--${props.theme}`,
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
    @update:visible="handleVisibleChange"
  >
    <slot />
    <template v-if="$slots.footer" #footer>
      <slot name="footer" />
    </template>
  </Dialog>
</template>

<style>
/* Theme variables for content inside PrimeVue Dialog */
.mtchat-dialog--light {
  --mtchat-bg: #ffffff;
  --mtchat-text: #1e293b;
  --mtchat-text-secondary: #64748b;
  --mtchat-border: #e2e8f0;
  --mtchat-bg-hover: #f1f5f9;
  --mtchat-hover: #f1f5f9;
  --mtchat-primary: #3B82F6;
  --mtchat-input-bg: #ffffff;
  --mtchat-input-border: #d1d5db;
}

.mtchat-dialog--dark {
  --mtchat-bg: #1f2937;
  --mtchat-text: #f8fafc;
  --mtchat-text-secondary: #94a3b8;
  --mtchat-border: #374151;
  --mtchat-bg-hover: #374151;
  --mtchat-hover: #374151;
  --mtchat-primary: #60a5fa;
  --mtchat-input-bg: #111827;
  --mtchat-input-border: #374151;
}
</style>
