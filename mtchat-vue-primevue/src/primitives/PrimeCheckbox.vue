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
  <div class="prime-checkbox">
    <Checkbox
      :model-value="modelValue"
      :disabled="disabled"
      :name="name"
      :binary="true"
      @update:model-value="handleChange"
    />
    <label v-if="label" class="prime-checkbox__label">{{ label }}</label>
  </div>
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
