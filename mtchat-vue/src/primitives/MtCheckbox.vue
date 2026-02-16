<script setup lang="ts">
/**
 * MtCheckbox - Native HTML checkbox primitive
 */

import type { MtCheckboxProps, MtCheckboxEmits } from '../registry/types'

withDefaults(defineProps<MtCheckboxProps>(), {
  disabled: false,
})

const emit = defineEmits<MtCheckboxEmits>()

function handleChange(e: Event) {
  const target = e.target as HTMLInputElement
  emit('update:modelValue', target.checked)
}
</script>

<template>
  <label class="mt-checkbox" :class="{ 'mt-checkbox--disabled': disabled }">
    <input
      type="checkbox"
      :checked="modelValue"
      :disabled="disabled"
      :name="name"
      class="mt-checkbox__input"
      @change="handleChange"
    />
    <span class="mt-checkbox__box">
      <svg
        v-if="modelValue"
        class="mt-checkbox__check"
        width="12"
        height="12"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="3"
        stroke-linecap="round"
        stroke-linejoin="round"
      >
        <polyline points="20 6 9 17 4 12" />
      </svg>
    </span>
    <span v-if="label" class="mt-checkbox__label">{{ label }}</span>
  </label>
</template>

<style scoped>
.mt-checkbox {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
  user-select: none;
}

.mt-checkbox--disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.mt-checkbox__input {
  position: absolute;
  opacity: 0;
  width: 0;
  height: 0;
}

.mt-checkbox__box {
  width: 18px;
  height: 18px;
  border: 2px solid var(--mtchat-border, #d1d5db);
  border-radius: 4px;
  background: var(--mtchat-bg, #ffffff);
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.15s ease;
  flex-shrink: 0;
}

.mt-checkbox__input:checked + .mt-checkbox__box {
  background: var(--mtchat-primary, #3b82f6);
  border-color: var(--mtchat-primary, #3b82f6);
}

.mt-checkbox__input:focus-visible + .mt-checkbox__box {
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.2);
}

.mt-checkbox__check {
  color: white;
}

.mt-checkbox__label {
  font-size: 14px;
  color: var(--mtchat-text, #1e293b);
}

.mt-checkbox:hover:not(.mt-checkbox--disabled) .mt-checkbox__box {
  border-color: var(--mtchat-primary, #3b82f6);
}
</style>
