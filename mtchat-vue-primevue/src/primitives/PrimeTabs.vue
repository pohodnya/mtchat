<script setup lang="ts">
/**
 * PrimeTabs - PrimeVue Tabs adapter
 */

import Tabs from 'primevue/tabs'
import TabList from 'primevue/tablist'
import type { MtTabsProps, MtTabsEmits } from '@mtchat/vue'

defineProps<MtTabsProps>()
const emit = defineEmits<MtTabsEmits>()

function handleChange(value: string | number) {
  emit('update:modelValue', String(value))
}
</script>

<template>
  <Tabs :value="modelValue" @update:value="handleChange">
    <TabList>
      <slot />
    </TabList>
  </Tabs>
</template>
