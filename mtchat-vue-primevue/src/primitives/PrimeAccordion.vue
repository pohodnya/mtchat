<script setup lang="ts">
/**
 * PrimeAccordion - PrimeVue Accordion adapter
 */

import Accordion from 'primevue/accordion'
import type { MtAccordionProps, MtAccordionEmits } from '@mtchat/vue'

withDefaults(defineProps<MtAccordionProps>(), {
  multiple: false,
})

const emit = defineEmits<MtAccordionEmits>()

function handleChange(value: string | string[] | null | undefined) {
  if (value !== null && value !== undefined) {
    emit('update:modelValue', value)
  }
}
</script>

<template>
  <Accordion
    :value="modelValue"
    :multiple="multiple"
    @update:value="handleChange"
  >
    <slot />
  </Accordion>
</template>
