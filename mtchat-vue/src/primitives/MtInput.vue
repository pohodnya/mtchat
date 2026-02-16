<script setup lang="ts">
/**
 * MtInput - Native HTML input primitive
 *
 * Text input with optional clear button
 */

import { ref } from 'vue'
import type { MtInputProps, MtInputEmits, MtInputExpose } from '../registry/types'

withDefaults(defineProps<MtInputProps>(), {
  type: 'text',
  disabled: false,
  invalid: false,
  clearable: false,
  size: 'md',
})

const emit = defineEmits<MtInputEmits>()

const inputRef = ref<HTMLInputElement | null>(null)

function handleInput(e: Event) {
  const target = e.target as HTMLInputElement
  emit('update:modelValue', target.value)
}

function handleClear() {
  emit('update:modelValue', '')
  emit('clear')
  inputRef.value?.focus()
}

function focus() {
  inputRef.value?.focus()
}

function select() {
  inputRef.value?.select()
}

defineExpose<MtInputExpose>({
  focus,
  select,
})
</script>

<template>
  <div
    class="mt-input-wrapper"
    :class="[
      `mt-input-wrapper--${size}`,
      {
        'mt-input-wrapper--disabled': disabled,
        'mt-input-wrapper--invalid': invalid,
      }
    ]"
  >
    <input
      ref="inputRef"
      :type="type"
      :value="modelValue"
      :placeholder="placeholder"
      :disabled="disabled"
      class="mt-input"
      @input="handleInput"
      @focus="emit('focus', $event)"
      @blur="emit('blur', $event)"
      @keydown="emit('keydown', $event)"
    />
    <button
      v-if="clearable && modelValue"
      class="mt-input__clear"
      type="button"
      tabindex="-1"
      @click="handleClear"
    >
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <line x1="18" y1="6" x2="6" y2="18" />
        <line x1="6" y1="6" x2="18" y2="18" />
      </svg>
    </button>
  </div>
</template>

<style scoped>
.mt-input-wrapper {
  position: relative;
  display: flex;
  align-items: center;
}

.mt-input {
  width: 100%;
  border: 1px solid var(--mtchat-border, #e2e8f0);
  border-radius: var(--mtchat-border-radius, 6px);
  background: var(--mtchat-bg, #ffffff);
  color: var(--mtchat-text, #1e293b);
  font-family: inherit;
  transition: border-color 0.15s, box-shadow 0.15s;
}

/* Sizes */
.mt-input-wrapper--sm .mt-input {
  padding: 6px 10px;
  font-size: 12px;
}

.mt-input-wrapper--md .mt-input {
  padding: 8px 12px;
  font-size: 14px;
}

.mt-input-wrapper--lg .mt-input {
  padding: 12px 16px;
  font-size: 16px;
}

/* Add right padding for clear button */
.mt-input-wrapper .mt-input {
  padding-right: 32px;
}

.mt-input:focus {
  outline: none;
  border-color: var(--mtchat-primary, #3b82f6);
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
}

.mt-input::placeholder {
  color: var(--mtchat-text-secondary, #94a3b8);
}

/* States */
.mt-input-wrapper--disabled .mt-input {
  opacity: 0.5;
  cursor: not-allowed;
  background: var(--mtchat-bg-secondary, #f8fafc);
}

.mt-input-wrapper--invalid .mt-input {
  border-color: #ef4444;
}

.mt-input-wrapper--invalid .mt-input:focus {
  box-shadow: 0 0 0 3px rgba(239, 68, 68, 0.1);
}

/* Clear button */
.mt-input__clear {
  position: absolute;
  right: 8px;
  padding: 4px;
  background: none;
  border: none;
  cursor: pointer;
  color: var(--mtchat-text-secondary, #94a3b8);
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 4px;
  transition: color 0.15s, background-color 0.15s;
}

.mt-input__clear:hover {
  color: var(--mtchat-text, #1e293b);
  background: var(--mtchat-bg-hover, #f1f5f9);
}
</style>
