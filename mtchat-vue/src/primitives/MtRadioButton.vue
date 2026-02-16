<script setup lang="ts">
/**
 * MtRadioButton - Native HTML radio button primitive
 */

import type { MtRadioButtonProps, MtRadioButtonEmits } from '../registry/types'

const props = withDefaults(defineProps<MtRadioButtonProps>(), {
  disabled: false,
})

const emit = defineEmits<MtRadioButtonEmits>()

function handleChange() {
  emit('update:modelValue', props.value)
}
</script>

<template>
  <label class="mt-radio" :class="{ 'mt-radio--disabled': disabled }">
    <input
      type="radio"
      :checked="modelValue === value"
      :disabled="disabled"
      :name="name"
      :value="value"
      class="mt-radio__input"
      @change="handleChange"
    />
    <span class="mt-radio__circle">
      <span v-if="modelValue === value" class="mt-radio__dot" />
    </span>
    <span v-if="label" class="mt-radio__label">{{ label }}</span>
  </label>
</template>

<style scoped>
.mt-radio {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
  user-select: none;
}

.mt-radio--disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.mt-radio__input {
  position: absolute;
  opacity: 0;
  width: 0;
  height: 0;
}

.mt-radio__circle {
  width: 18px;
  height: 18px;
  border: 2px solid var(--mtchat-border, #d1d5db);
  border-radius: 50%;
  background: var(--mtchat-bg, #ffffff);
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.15s ease;
  flex-shrink: 0;
}

.mt-radio__input:checked + .mt-radio__circle {
  border-color: var(--mtchat-primary, #3b82f6);
}

.mt-radio__input:focus-visible + .mt-radio__circle {
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.2);
}

.mt-radio__dot {
  width: 10px;
  height: 10px;
  background: var(--mtchat-primary, #3b82f6);
  border-radius: 50%;
}

.mt-radio__label {
  font-size: 14px;
  color: var(--mtchat-text, #1e293b);
}

.mt-radio:hover:not(.mt-radio--disabled) .mt-radio__circle {
  border-color: var(--mtchat-primary, #3b82f6);
}
</style>
