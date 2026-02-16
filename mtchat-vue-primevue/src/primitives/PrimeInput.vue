<script setup lang="ts">
/**
 * PrimeInput - PrimeVue InputText adapter
 */

import { ref } from 'vue'
import InputText from 'primevue/inputtext'
import type { MtInputProps, MtInputEmits, MtInputExpose } from '@mtchat/vue'

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
  focus()
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
  <div class="prime-input-wrapper">
    <InputText
      ref="inputRef"
      :type="type"
      :value="modelValue"
      :placeholder="placeholder"
      :disabled="disabled"
      :invalid="invalid"
      class="prime-input"
      @input="handleInput"
      @focus="emit('focus', $event)"
      @blur="emit('blur', $event)"
      @keydown="emit('keydown', $event)"
    />
    <button
      v-if="clearable && modelValue"
      class="prime-input__clear"
      type="button"
      tabindex="-1"
      @click="handleClear"
    >
      <i class="pi pi-times" />
    </button>
  </div>
</template>

<style scoped>
.prime-input-wrapper {
  position: relative;
  display: flex;
  align-items: center;
  width: 100%;
}

.prime-input {
  width: 100%;
}

.prime-input-wrapper .prime-input {
  padding-right: 32px;
}

.prime-input__clear {
  position: absolute;
  right: 8px;
  padding: 4px;
  background: none;
  border: none;
  cursor: pointer;
  color: var(--p-text-muted-color);
  display: flex;
  align-items: center;
  justify-content: center;
}

.prime-input__clear:hover {
  color: var(--p-text-color);
}
</style>
