<script setup lang="ts">
/**
 * PrimeTag - PrimeVue Tag adapter
 *
 * Always uses rounded=true as required by MTChatPrime design.
 */

import Tag from 'primevue/tag'
import type { MtTagProps } from '@mtchat/vue'

defineProps<MtTagProps>()
</script>

<template>
  <Tag
    :value="value"
    :rounded="true"
    severity="secondary"
    :pt="{ root: { style: 'padding: 3.5px 7px' }, label: { style: 'line-height: 100%' } }"
  />
</template>
