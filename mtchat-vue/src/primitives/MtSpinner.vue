<script setup lang="ts">
/**
 * MtSpinner - Native CSS loading spinner
 *
 * Default implementation, used when no registry is provided. Hosts on PrimeVue
 * get PrimeVue's ProgressSpinner instead (see @mtchat/vue-primevue).
 */

import type { MtSpinnerProps } from '../registry/types'

const props = withDefaults(defineProps<MtSpinnerProps>(), {
  size: 32,
})
</script>

<template>
  <span
    class="mt-spinner"
    role="status"
    :aria-label="props.label"
    :aria-busy="true"
    :style="{ width: `${props.size}px`, height: `${props.size}px` }"
  />
</template>

<style scoped>
.mt-spinner {
  display: inline-block;
  box-sizing: border-box;
  border: 2px solid var(--mtchat-border, #e2e8f0);
  border-top-color: var(--mtchat-primary, #3b82f6);
  border-radius: 50%;
  animation: mt-spinner-rotate 0.7s linear infinite;
}

@keyframes mt-spinner-rotate {
  to {
    transform: rotate(360deg);
  }
}

/* Keep a visible indicator without motion for users who ask for less of it */
@media (prefers-reduced-motion: reduce) {
  .mt-spinner {
    animation-duration: 2.5s;
  }
}
</style>
