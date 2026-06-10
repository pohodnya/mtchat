<script setup lang="ts">
/**
 * MtTag - Native HTML tag primitive
 */

import type { MtTagProps } from '../registry/types'

defineProps<MtTagProps>()
</script>

<template>
  <span class="mt-tag">{{ value }}</span>
</template>

<style scoped>
.mt-tag {
  display: inline-flex;
  align-items: center;
  padding: 3.5px 7px;
  border-radius: 6px;
  font-size: 0.875rem;
  font-weight: 700;
  line-height: 100%;
  background: var(--mtchat-bg-secondary, #f1f5f9);
  color: var(--mtchat-text, #1e293b);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 100%;
}
</style>
