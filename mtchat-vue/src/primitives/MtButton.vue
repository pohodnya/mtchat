<script setup lang="ts">
/**
 * MtButton - Native HTML button primitive
 *
 * Provides a styled button with variants (primary, secondary, danger, ghost, text)
 */

import type { MtButtonProps } from '../registry/types'

withDefaults(defineProps<MtButtonProps>(), {
  variant: 'secondary',
  size: 'md',
  disabled: false,
  loading: false,
  type: 'button',
  icon: false,
})

const emit = defineEmits<{
  click: [event: MouseEvent]
}>()
</script>

<template>
  <button
    :type="type"
    :disabled="disabled || loading"
    :title="title"
    :class="[
      'mt-button',
      `mt-button--${variant}`,
      `mt-button--${size}`,
      {
        'mt-button--loading': loading,
        'mt-button--icon': icon,
      }
    ]"
    @click="emit('click', $event)"
  >
    <span v-if="loading" class="mt-button__spinner"></span>
    <slot />
  </button>
</template>

<style scoped>
.mt-button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  border: none;
  border-radius: var(--mtchat-border-radius, 6px);
  font-family: inherit;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s ease;
  white-space: nowrap;
}

/* Sizes */
.mt-button--sm {
  padding: 6px 12px;
  font-size: 12px;
  min-height: 28px;
}

.mt-button--md {
  padding: 8px 16px;
  font-size: 14px;
  min-height: 36px;
}

.mt-button--lg {
  padding: 12px 24px;
  font-size: 16px;
  min-height: 44px;
}

/* Icon-only buttons */
.mt-button--icon.mt-button--sm {
  padding: 6px;
  min-width: 28px;
}

.mt-button--icon.mt-button--md {
  padding: 8px;
  min-width: 36px;
}

.mt-button--icon.mt-button--lg {
  padding: 12px;
  min-width: 44px;
}

/* Variants */
.mt-button--primary {
  background: var(--mtchat-primary);
  color: white;
}

.mt-button--primary:hover:not(:disabled) {
  opacity: 0.9;
}

.mt-button--secondary {
  background: var(--mtchat-bg-secondary);
  color: var(--mtchat-text);
}

.mt-button--secondary:hover:not(:disabled) {
  background: var(--mtchat-bg-hover);
}

.mt-button--danger {
  background: #ef4444;
  color: white;
}

.mt-button--danger:hover:not(:disabled) {
  background: #dc2626;
}

.mt-button--ghost {
  background: transparent;
  color: var(--mtchat-text-secondary);
}

.mt-button--ghost:hover:not(:disabled) {
  background: var(--mtchat-bg-hover);
  color: var(--mtchat-text);
}

.mt-button--text {
  background: transparent;
  color: var(--mtchat-primary);
  padding-left: 4px;
  padding-right: 4px;
}

.mt-button--text:hover:not(:disabled) {
  text-decoration: underline;
}

/* States */
.mt-button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.mt-button--loading {
  position: relative;
  color: transparent !important;
}

/* Spinner */
.mt-button__spinner {
  position: absolute;
  width: 16px;
  height: 16px;
  border: 2px solid currentColor;
  border-right-color: transparent;
  border-radius: 50%;
  animation: mt-spin 0.75s linear infinite;
}

.mt-button--primary .mt-button__spinner,
.mt-button--danger .mt-button__spinner {
  border-color: rgba(255, 255, 255, 0.3);
  border-right-color: white;
}

@keyframes mt-spin {
  to {
    transform: rotate(360deg);
  }
}
</style>
