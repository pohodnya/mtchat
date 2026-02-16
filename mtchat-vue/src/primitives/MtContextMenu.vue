<script setup lang="ts">
/**
 * MtContextMenu - Native HTML context menu primitive
 *
 * Context menu shown on right-click, positioned at cursor
 */

import { ref, onMounted, onUnmounted } from 'vue'
import type { MtContextMenuProps, MtContextMenuEmits, MtContextMenuExpose, MtMenuItem } from '../registry/types'

defineProps<MtContextMenuProps>()

const emit = defineEmits<MtContextMenuEmits>()

const visible = ref(false)
const menuRef = ref<HTMLElement | null>(null)
const position = ref({ top: 0, left: 0 })

function show(event: MouseEvent) {
  event.preventDefault()

  const menuWidth = 160
  const menuHeight = 150 // Approximate

  let x = event.clientX
  let y = event.clientY

  // Adjust if menu goes off screen
  if (x + menuWidth > window.innerWidth) {
    x = window.innerWidth - menuWidth - 8
  }
  if (y + menuHeight > window.innerHeight) {
    y = window.innerHeight - menuHeight - 8
  }

  position.value = { top: Math.max(8, y), left: Math.max(8, x) }
  visible.value = true
}

function hide() {
  visible.value = false
  emit('hide')
}

function handleItemClick(item: MtMenuItem) {
  if (item.disabled) return

  if (item.command) {
    item.command()
  }
  emit('select', item)
  hide()
}

function handleOutsideClick(e: MouseEvent) {
  if (visible.value && menuRef.value && !menuRef.value.contains(e.target as Node)) {
    hide()
  }
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape' && visible.value) {
    hide()
  }
}

onMounted(() => {
  document.addEventListener('click', handleOutsideClick)
  document.addEventListener('keydown', handleKeydown)
})

onUnmounted(() => {
  document.removeEventListener('click', handleOutsideClick)
  document.removeEventListener('keydown', handleKeydown)
})

defineExpose<MtContextMenuExpose>({
  show,
  hide,
})
</script>

<template>
  <Teleport to="body">
    <Transition name="mt-context-menu">
      <div
        v-if="visible"
        ref="menuRef"
        class="mt-context-menu"
        :style="{ top: `${position.top}px`, left: `${position.left}px` }"
        @click.stop
      >
        <template v-for="(item, index) in items" :key="index">
          <div v-if="item.separator" class="mt-context-menu__separator" />
          <button
            v-else
            class="mt-context-menu__item"
            :class="{
              'mt-context-menu__item--disabled': item.disabled,
              'mt-context-menu__item--danger': item.danger,
            }"
            :disabled="item.disabled"
            @click="handleItemClick(item)"
          >
            <slot name="item-icon" :item="item">
              <span v-if="item.icon" class="mt-context-menu__icon">{{ item.icon }}</span>
            </slot>
            <span class="mt-context-menu__label">{{ item.label }}</span>
          </button>
        </template>
      </div>
    </Transition>
  </Teleport>
</template>

<style>
.mt-context-menu {
  position: fixed;
  background: var(--mtchat-bg, #ffffff);
  border: 1px solid var(--mtchat-border, #e2e8f0);
  border-radius: 8px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  z-index: 1000;
  min-width: 160px;
  padding: 4px;
}

.mt-context-menu__separator {
  height: 1px;
  background: var(--mtchat-border, #e2e8f0);
  margin: 4px 0;
}

.mt-context-menu__item {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 8px 12px;
  background: none;
  border: none;
  font-size: 13px;
  font-family: inherit;
  color: var(--mtchat-text, #1e293b);
  cursor: pointer;
  border-radius: 4px;
  text-align: left;
  transition: background-color 0.15s;
}

.mt-context-menu__item:hover:not(:disabled) {
  background: var(--mtchat-bg-hover, #f1f5f9);
}

.mt-context-menu__item--disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.mt-context-menu__item--danger {
  color: #ef4444;
}

.mt-context-menu__icon {
  flex-shrink: 0;
  width: 16px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.mt-context-menu__label {
  flex: 1;
}

/* Transitions */
.mt-context-menu-enter-active,
.mt-context-menu-leave-active {
  transition: opacity 0.1s ease, transform 0.1s ease;
}

.mt-context-menu-enter-from,
.mt-context-menu-leave-to {
  opacity: 0;
  transform: scale(0.95);
}
</style>
