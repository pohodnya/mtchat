<script setup lang="ts">
/**
 * PrimeSpinner - PrimeVue ProgressSpinner adapter
 *
 * ProgressSpinner has no size prop: it fills its container and is sized through
 * style, so the registry's `size` is mapped onto width/height here.
 */

import ProgressSpinner from 'primevue/progressspinner'
import type { MtSpinnerProps } from '@mtchat/vue'

const props = withDefaults(defineProps<MtSpinnerProps>(), {
  size: 32,
})

/**
 * ProgressSpinner ships a second animation, p-progressspinner-color, that
 * repaints the stroke through four theme colors (red, blue, green, yellow in
 * Aura) on a 6s loop. A chat spinner cycling through the rainbow is not what
 * MTChat wants, so pin the stroke to the accent colour instead.
 *
 * !important is required, not sloppiness: a plain declaration loses to the
 * keyframes, while an important one outranks them. Overriding `animation`
 * would work too but would hard-code PrimeVue's internal keyframe name and
 * silently kill the spin if they ever rename it.
 */
const CIRCLE_STYLE = 'stroke: var(--mtchat-primary, #6366f1) !important'
</script>

<template>
  <ProgressSpinner
    :style="{ width: `${props.size}px`, height: `${props.size}px` }"
    strokeWidth="4"
    :aria-label="props.label"
    :pt="{ circle: { style: CIRCLE_STYLE } }"
  />
</template>
