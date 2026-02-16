<script setup lang="ts">
/**
 * MtAccordion - Native HTML accordion container primitive
 *
 * Works together with MtAccordionPanel for collapsible sections
 */

import { provide, computed } from 'vue'
import type { MtAccordionProps, MtAccordionEmits } from '../registry/types'
import { ACCORDION_KEY } from './keys'

const props = withDefaults(defineProps<MtAccordionProps>(), {
  multiple: false,
})

const emit = defineEmits<MtAccordionEmits>()

const expandedPanels = computed(() => {
  if (!props.modelValue) return []
  return Array.isArray(props.modelValue) ? props.modelValue : [props.modelValue]
})

function isExpanded(value: string): boolean {
  return expandedPanels.value.includes(value)
}

function toggle(value: string) {
  if (props.multiple) {
    const newValue = isExpanded(value)
      ? expandedPanels.value.filter(v => v !== value)
      : [...expandedPanels.value, value]
    emit('update:modelValue', newValue)
  } else {
    emit('update:modelValue', isExpanded(value) ? '' : value)
  }
}

provide(ACCORDION_KEY, {
  isExpanded,
  toggle,
})
</script>

<template>
  <div class="mt-accordion">
    <slot />
  </div>
</template>

<style scoped>
.mt-accordion {
  display: flex;
  flex-direction: column;
  border: 1px solid var(--mtchat-border, #e2e8f0);
  border-radius: var(--mtchat-border-radius, 6px);
  overflow: hidden;
}

.mt-accordion :deep(.mt-accordion-panel + .mt-accordion-panel) {
  border-top: 1px solid var(--mtchat-border, #e2e8f0);
}
</style>
