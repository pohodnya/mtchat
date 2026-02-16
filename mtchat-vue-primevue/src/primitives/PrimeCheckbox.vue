<script setup lang="ts">
/**
 * PrimeCheckbox - PrimeVue Checkbox adapter
 */

import Checkbox from 'primevue/checkbox'
import type { MtCheckboxProps, MtCheckboxEmits } from '@mtchat/vue'

withDefaults(defineProps<MtCheckboxProps>(), {
  disabled: false,
})

const emit = defineEmits<MtCheckboxEmits>()

function handleChange(value: boolean) {
  emit('update:modelValue', value)
}
</script>

<template>
  <span class="prime-checkbox">
    <Checkbox
      :model-value="modelValue"
      :disabled="disabled"
      :name="name"
      :binary="true"
      @update:model-value="handleChange"
    />
    <span v-if="label" class="prime-checkbox__label">{{ label }}</span>
  </span>
</template>

<style scoped>
.prime-checkbox {
  display: inline-flex;
  align-items: center;
  gap: 8px;
}

.prime-checkbox__label {
  font-size: 14px;
  cursor: pointer;
}
</style>
