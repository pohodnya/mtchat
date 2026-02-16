<script setup lang="ts">
/**
 * MtTabs - Native HTML tabs container primitive
 *
 * Works together with MtTab for tab navigation
 */

import { provide } from 'vue'
import type { MtTabsProps, MtTabsEmits } from '../registry/types'
import { TABS_KEY } from './keys'

const props = defineProps<MtTabsProps>()
const emit = defineEmits<MtTabsEmits>()

// Provide tabs context to children
provide(TABS_KEY, {
  activeValue: () => props.modelValue,
  setActiveValue: (value: string) => emit('update:modelValue', value),
})
</script>

<template>
  <div class="mt-tabs">
    <div class="mt-tabs__header">
      <slot />
    </div>
  </div>
</template>

<style scoped>
.mt-tabs {
  display: flex;
  flex-direction: column;
}

.mt-tabs__header {
  display: flex;
  border-bottom: 1px solid var(--mtchat-border, #e2e8f0);
}
</style>
