<script setup lang="ts">
/**
 * PrimeTab - PrimeVue Tab adapter
 */

import Tab from 'primevue/tab'
import type { MtTabProps } from '@mtchat/vue'

withDefaults(defineProps<MtTabProps>(), {
  disabled: false,
})
</script>

<template>
  <Tab :value="value" :disabled="disabled">
    {{ label }}
    <span v-if="badge !== undefined && badge !== null" class="prime-tab__badge">
      {{ badge }}
    </span>
  </Tab>
</template>

<style scoped>
.prime-tab__badge {
  margin-left: 6px;
  padding: 2px 6px;
  background: var(--p-surface-200);
  border-radius: 10px;
  font-size: 11px;
}
</style>
