<script setup lang="ts">
/**
 * MtAccordionPanel - Native HTML accordion panel primitive
 *
 * Used inside MtAccordion container
 */

import { inject, computed } from 'vue'
import type { MtAccordionPanelProps } from '../registry/types'
import { ACCORDION_KEY } from './keys'

const props = withDefaults(defineProps<MtAccordionPanelProps>(), {
  disabled: false,
})

const accordion = inject(ACCORDION_KEY)

const isExpanded = computed(() => accordion?.isExpanded(props.value) ?? false)

function handleToggle() {
  if (!props.disabled && accordion) {
    accordion.toggle(props.value)
  }
}
</script>

<template>
  <div
    class="mt-accordion-panel"
    :class="{
      'mt-accordion-panel--expanded': isExpanded,
      'mt-accordion-panel--disabled': disabled,
    }"
  >
    <button
      type="button"
      class="mt-accordion-panel__header"
      :disabled="disabled"
      @click="handleToggle"
    >
      <svg
        class="mt-accordion-panel__icon"
        width="12"
        height="12"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
      >
        <polyline points="9 18 15 12 9 6" />
      </svg>
      <span class="mt-accordion-panel__title">{{ header }}</span>
    </button>

    <Transition name="mt-accordion-panel">
      <div v-if="isExpanded" class="mt-accordion-panel__content">
        <slot />
      </div>
    </Transition>
  </div>
</template>

<style scoped>
.mt-accordion-panel__header {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 12px 16px;
  background: var(--mtchat-bg-secondary, #f8fafc);
  border: none;
  font-size: 14px;
  font-weight: 500;
  font-family: inherit;
  color: var(--mtchat-text, #1e293b);
  cursor: pointer;
  text-align: left;
  transition: background-color 0.15s;
}

.mt-accordion-panel__header:hover:not(:disabled) {
  background: var(--mtchat-bg-hover, #f1f5f9);
}

.mt-accordion-panel--disabled .mt-accordion-panel__header {
  opacity: 0.5;
  cursor: not-allowed;
}

.mt-accordion-panel__icon {
  flex-shrink: 0;
  color: var(--mtchat-text-secondary, #64748b);
  transition: transform 0.2s ease;
}

.mt-accordion-panel--expanded .mt-accordion-panel__icon {
  transform: rotate(90deg);
}

.mt-accordion-panel__title {
  flex: 1;
}

.mt-accordion-panel__content {
  padding: 0;
  overflow: hidden;
}

/* Transition */
.mt-accordion-panel-enter-active,
.mt-accordion-panel-leave-active {
  transition: all 0.2s ease;
  overflow: hidden;
}

.mt-accordion-panel-enter-from,
.mt-accordion-panel-leave-to {
  max-height: 0;
  opacity: 0;
}

.mt-accordion-panel-enter-to,
.mt-accordion-panel-leave-from {
  max-height: 1000px;
  opacity: 1;
}
</style>
