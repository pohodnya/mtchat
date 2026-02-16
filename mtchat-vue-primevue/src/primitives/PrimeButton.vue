<script setup lang="ts">
/**
 * PrimeButton - PrimeVue Button adapter
 */

import Button from 'primevue/button'
import type { MtButtonProps } from '@mtchat/vue'

const props = withDefaults(defineProps<MtButtonProps>(), {
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

// Map variant to PrimeVue severity
const severityMap: Record<string, string> = {
  primary: '',
  secondary: 'secondary',
  danger: 'danger',
  ghost: 'secondary',
  text: 'secondary',
}

// Map size to PrimeVue size
const sizeMap: Record<string, string> = {
  sm: 'small',
  md: '',
  lg: 'large',
}

const severity = severityMap[props.variant] || 'secondary'
const size = sizeMap[props.size] || ''
const text = props.variant === 'text'
const outlined = props.variant === 'ghost'
</script>

<template>
  <Button
    :type="type"
    :disabled="disabled"
    :loading="loading"
    :severity="severity || undefined"
    :size="size || undefined"
    :text="text"
    :outlined="outlined"
    :aria-label="title"
    v-tooltip.top="title"
    @click="emit('click', $event)"
  >
    <slot />
  </Button>
</template>
