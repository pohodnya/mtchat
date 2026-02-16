<script setup lang="ts">
/**
 * MtMenu - Native HTML popup menu primitive
 *
 * Positioned popup menu triggered by a button or other element
 */

import { ref, onMounted, onUnmounted, nextTick } from 'vue'
import type { MtMenuProps, MtMenuEmits, MtMenuExpose, MtMenuItem } from '../registry/types'

const props = withDefaults(defineProps<MtMenuProps>(), {
  popup: true,
})

const emit = defineEmits<MtMenuEmits>()

const visible = ref(false)
const menuRef = ref<HTMLElement | null>(null)
const position = ref({ top: 0, left: 0 })

function toggle(event: Event) {
  if (visible.value) {
    hide()
  } else {
    show(event)
  }
}

function show(event: Event) {
  visible.value = true

  nextTick(() => {
    if (props.popup && event.target) {
      const target = event.target as HTMLElement
      const rect = target.getBoundingClientRect()
      const menu = menuRef.value

      if (menu) {
        const menuRect = menu.getBoundingClientRect()
        let top = rect.bottom + 4
        let left = rect.left

        // Adjust if menu goes off screen
        if (left + menuRect.width > window.innerWidth) {
          left = window.innerWidth - menuRect.width - 8
        }
        if (top + menuRect.height > window.innerHeight) {
          top = rect.top - menuRect.height - 4
        }

        position.value = { top, left: Math.max(8, left) }
      }
    }
  })
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
  if (menuRef.value && !menuRef.value.contains(e.target as Node)) {
    hide()
  }
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') {
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

defineExpose<MtMenuExpose>({
  toggle,
  show,
  hide,
})
</script>

<template>
  <Teleport v-if="popup" to="body">
    <Transition name="mt-menu">
      <div
        v-if="visible"
        ref="menuRef"
        class="mt-menu"
        :style="{ top: `${position.top}px`, left: `${position.left}px` }"
        @click.stop
      >
        <template v-for="(item, index) in items" :key="index">
          <div v-if="item.separator" class="mt-menu__separator" />
          <button
            v-else
            class="mt-menu__item"
            :class="{
              'mt-menu__item--disabled': item.disabled,
              'mt-menu__item--danger': item.danger,
            }"
            :disabled="item.disabled"
            @click="handleItemClick(item)"
          >
            <slot name="item-icon" :item="item">
              <span v-if="item.icon" class="mt-menu__icon">{{ item.icon }}</span>
            </slot>
            <span class="mt-menu__label">{{ item.label }}</span>
          </button>
        </template>
      </div>
    </Transition>
  </Teleport>

  <div v-else ref="menuRef" class="mt-menu mt-menu--inline">
    <template v-for="(item, index) in items" :key="index">
      <div v-if="item.separator" class="mt-menu__separator" />
      <button
        v-else
        class="mt-menu__item"
        :class="{
          'mt-menu__item--disabled': item.disabled,
          'mt-menu__item--danger': item.danger,
        }"
        :disabled="item.disabled"
        @click="handleItemClick(item)"
      >
        <slot name="item-icon" :item="item">
          <span v-if="item.icon" class="mt-menu__icon">{{ item.icon }}</span>
        </slot>
        <span class="mt-menu__label">{{ item.label }}</span>
      </button>
    </template>
  </div>
</template>

<style>
.mt-menu {
  position: fixed;
  background: var(--mtchat-bg, #ffffff);
  border: 1px solid var(--mtchat-border, #e2e8f0);
  border-radius: 8px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  z-index: 1000;
  min-width: 160px;
  padding: 4px;
}

.mt-menu--inline {
  position: static;
  box-shadow: none;
  border: none;
  padding: 0;
  min-width: auto;
}

.mt-menu__separator {
  height: 1px;
  background: var(--mtchat-border, #e2e8f0);
  margin: 4px 0;
}

.mt-menu__item {
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

.mt-menu__item:hover:not(:disabled) {
  background: var(--mtchat-bg-hover, #f1f5f9);
}

.mt-menu__item--disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.mt-menu__item--danger {
  color: #ef4444;
}

.mt-menu__icon {
  flex-shrink: 0;
  width: 16px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.mt-menu__label {
  flex: 1;
}

/* Transitions */
.mt-menu-enter-active,
.mt-menu-leave-active {
  transition: opacity 0.15s ease, transform 0.15s ease;
}

.mt-menu-enter-from,
.mt-menu-leave-to {
  opacity: 0;
  transform: translateY(-4px);
}
</style>
