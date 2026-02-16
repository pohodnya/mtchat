<script setup lang="ts">
/**
 * PrimeButton - PrimeVue Button adapter
 *
 * Maps MtButton variants to PrimeVue Button props:
 * - primary → default (no severity)
 * - secondary → severity="secondary"
 * - danger → severity="danger"
 * - ghost → outlined
 * - text → text
 */

import { computed } from 'vue'
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

const buttonProps = computed(() => {
  const base: Record<string, any> = {}

  // Size
  if (props.size === 'sm') base.size = 'small'
  if (props.size === 'lg') base.size = 'large'

  // Variant
  switch (props.variant) {
    case 'primary':
      // Default PrimeVue button is primary
      break
    case 'secondary':
      base.severity = 'secondary'
      break
    case 'danger':
      base.severity = 'danger'
      break
    case 'ghost':
      base.outlined = true
      break
    case 'text':
      base.text = true
      break
  }

  return base
})
</script>

<template>
  <Button
    :type="type"
    :disabled="disabled"
    :loading="loading"
    v-bind="buttonProps"
    :aria-label="title"
    v-tooltip.top="title"
    @click="emit('click', $event)"
  >
    <slot />
  </Button>
</template>
