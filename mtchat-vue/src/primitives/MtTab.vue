<script setup lang="ts">
/**
 * MtTab - Native HTML tab button primitive
 *
 * Used inside MtTabs container
 */

import { inject, computed } from 'vue'
import type { MtTabProps } from '../registry/types'
import { TABS_KEY } from './keys'

const props = withDefaults(defineProps<MtTabProps>(), {
  disabled: false,
})

const tabs = inject(TABS_KEY)

const isActive = computed(() => tabs?.activeValue() === props.value)

function handleClick() {
  if (!props.disabled && tabs) {
    tabs.setActiveValue(props.value)
  }
}
</script>

<template>
  <button
    type="button"
    class="mt-tab"
    :class="{
      'mt-tab--active': isActive,
      'mt-tab--disabled': disabled,
    }"
    :disabled="disabled"
    @click="handleClick"
  >
    <span class="mt-tab__label">{{ label }}</span>
    <span v-if="badge !== undefined && badge !== null" class="mt-tab__badge">
      {{ badge }}
    </span>
  </button>
</template>

<style scoped>
.mt-tab {
  flex: 1;
  padding: 12px 16px;
  background: none;
  border: none;
  border-bottom: 2px solid transparent;
  font-size: 14px;
  font-weight: 500;
  font-family: inherit;
  color: var(--mtchat-text-secondary, #64748b);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  transition: all 0.15s ease;
  margin-bottom: -1px;
}

.mt-tab:hover:not(:disabled) {
  background: var(--mtchat-bg-hover, #f1f5f9);
}

.mt-tab--active {
  color: var(--mtchat-primary, #3b82f6);
  border-bottom-color: var(--mtchat-primary, #3b82f6);
}

.mt-tab--disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.mt-tab__label {
  white-space: nowrap;
}

.mt-tab__badge {
  padding: 2px 6px;
  background: var(--mtchat-bg-secondary, #f1f5f9);
  border-radius: 10px;
  font-size: 11px;
  line-height: 1;
}

.mt-tab--active .mt-tab__badge {
  background: rgba(59, 130, 246, 0.1);
  color: var(--mtchat-primary, #3b82f6);
}
</style>
