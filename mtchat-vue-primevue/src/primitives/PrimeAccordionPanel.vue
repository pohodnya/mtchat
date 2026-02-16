<script setup lang="ts">
/**
 * PrimeAccordionPanel - PrimeVue AccordionPanel adapter
 */

import AccordionPanel from 'primevue/accordionpanel'
import AccordionHeader from 'primevue/accordionheader'
import AccordionContent from 'primevue/accordioncontent'
import type { MtAccordionPanelProps } from '@mtchat/vue'

withDefaults(defineProps<MtAccordionPanelProps>(), {
  disabled: false,
})
</script>

<template>
  <AccordionPanel :value="value" :disabled="disabled">
    <AccordionHeader>{{ header }}</AccordionHeader>
    <AccordionContent>
      <slot />
    </AccordionContent>
  </AccordionPanel>
</template>
