<script setup lang="ts">
/**
 * MtDialog - Native HTML dialog primitive
 *
 * Modal dialog with overlay, using Teleport for proper z-index stacking
 */

import { watch, onUnmounted } from 'vue'
import type { MtDialogProps, MtDialogEmits } from '../registry/types'

const props = withDefaults(defineProps<MtDialogProps>(), {
  modal: true,
  closable: true,
  maxWidth: '400px',
  draggable: false,
  theme: 'light',
})

const emit = defineEmits<MtDialogEmits>()

function close() {
  emit('update:visible', false)
  emit('close')
}

function handleOverlayClick(e: MouseEvent) {
  if (props.modal && e.target === e.currentTarget) {
    close()
  }
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape' && props.visible && props.closable) {
    close()
  }
}

watch(() => props.visible, (visible) => {
  if (visible) {
    document.addEventListener('keydown', handleKeydown)
    document.body.style.overflow = 'hidden'
  } else {
    document.removeEventListener('keydown', handleKeydown)
    document.body.style.overflow = ''
  }
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeydown)
  document.body.style.overflow = ''
})
</script>

<template>
  <Teleport to="body">
    <Transition name="mt-dialog">
      <div
        v-if="visible"
        :class="['mt-dialog-overlay', `mt-dialog--${theme}`]"
        @click="handleOverlayClick"
      >
        <div
          class="mt-dialog"
          :style="{ maxWidth }"
          role="dialog"
          aria-modal="true"
        >
          <!-- Header -->
          <div v-if="header || closable" class="mt-dialog__header">
            <h2 v-if="header" class="mt-dialog__title">{{ header }}</h2>
            <button
              v-if="closable"
              class="mt-dialog__close"
              type="button"
              aria-label="Close"
              @click="close"
            >
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <line x1="18" y1="6" x2="6" y2="18" />
                <line x1="6" y1="6" x2="18" y2="18" />
              </svg>
            </button>
          </div>

          <!-- Content -->
          <div class="mt-dialog__content">
            <slot />
          </div>

          <!-- Footer (optional slot) -->
          <div v-if="$slots.footer" class="mt-dialog__footer">
            <slot name="footer" />
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style>
/* Theme variables */
.mt-dialog--light {
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

.mt-dialog--dark {
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

.mt-dialog-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  padding: 16px;
}

.mt-dialog {
  background: var(--mtchat-bg, #ffffff);
  border-radius: 12px;
  width: 100%;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.2);
  color: var(--mtchat-text, #1e293b);
  display: flex;
  flex-direction: column;
  max-height: calc(100vh - 32px);
}

.mt-dialog__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 16px;
  border-bottom: 1px solid var(--mtchat-border, #e2e8f0);
  flex-shrink: 0;
}

.mt-dialog__title {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
}

.mt-dialog__close {
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  background: none;
  border-radius: 6px;
  cursor: pointer;
  color: var(--mtchat-text-secondary, #64748b);
  transition: background-color 0.2s, color 0.2s;
}

.mt-dialog__close:hover {
  background: var(--mtchat-bg-hover, #f1f5f9);
  color: var(--mtchat-text, #1e293b);
}

.mt-dialog__content {
  padding: 16px;
  overflow-y: auto;
  flex: 1;
}

.mt-dialog__footer {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding: 12px 16px;
  border-top: 1px solid var(--mtchat-border, #e2e8f0);
  flex-shrink: 0;
}

/* Transitions */
.mt-dialog-enter-active,
.mt-dialog-leave-active {
  transition: opacity 0.2s ease;
}

.mt-dialog-enter-active .mt-dialog,
.mt-dialog-leave-active .mt-dialog {
  transition: transform 0.2s ease, opacity 0.2s ease;
}

.mt-dialog-enter-from,
.mt-dialog-leave-to {
  opacity: 0;
}

.mt-dialog-enter-from .mt-dialog,
.mt-dialog-leave-to .mt-dialog {
  transform: scale(0.95);
  opacity: 0;
}
</style>
