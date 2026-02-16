<script setup lang="ts">
/**
 * PrimeRadioButton - PrimeVue RadioButton adapter
 */

import RadioButton from 'primevue/radiobutton'
import type { MtRadioButtonProps, MtRadioButtonEmits } from '@mtchat/vue'

const props = withDefaults(defineProps<MtRadioButtonProps>(), {
  disabled: false,
})

const emit = defineEmits<MtRadioButtonEmits>()

function handleChange() {
  emit('update:modelValue', props.value)
}
</script>

<template>
  <div class="prime-radio">
    <RadioButton
      :model-value="modelValue"
      :value="value"
      :disabled="disabled"
      :name="name"
      @update:model-value="handleChange"
    />
    <label v-if="label" class="prime-radio__label" @click="handleChange">{{ label }}</label>
  </div>
</template>

<style scoped>
.prime-radio {
  display: inline-flex;
  align-items: center;
  gap: 8px;
}

.prime-radio__label {
  font-size: 14px;
  cursor: pointer;
}
</style>
