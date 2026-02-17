<script setup lang="ts">
/**
 * PrimeContextMenu - PrimeVue ContextMenu adapter
 */

import { ref, computed } from 'vue'
import ContextMenu from 'primevue/contextmenu'
import type { MtContextMenuProps, MtContextMenuEmits, MtContextMenuExpose } from '@mtchat/vue'

const props = defineProps<MtContextMenuProps>()

const emit = defineEmits<MtContextMenuEmits>()

const menuRef = ref<InstanceType<typeof ContextMenu> | null>(null)

// Convert items to PrimeVue format
const primeItems = computed(() => {
  return props.items.map(item => {
    if (item.separator) {
      return { separator: true }
    }
    return {
      label: item.label,
      icon: item.icon,
      disabled: item.disabled,
      class: item.danger ? 'p-menuitem-danger' : undefined,
      command: () => {
        if (item.command) item.command()
        emit('select', item)
      },
    }
  })
})

const menuPt = computed(() => ({
  root: {
    class: props.theme ? `mtchat-contextmenu--${props.theme}` : null,
  },
}))

function show(event: MouseEvent) {
  menuRef.value?.show(event)
}

function hide() {
  menuRef.value?.hide()
  emit('hide')
}

defineExpose<MtContextMenuExpose>({
  show,
  hide,
})
</script>

<template>
  <ContextMenu
    ref="menuRef"
    :model="primeItems"
    :pt="menuPt"
    append-to="self"
    @hide="emit('hide')"
  >
    <template #item="{ item, props: itemProps }">
      <a v-bind="itemProps.action" :class="{ 'text-red-500': item.class === 'p-menuitem-danger' }">
        <slot name="item-icon" :item="item">
          <span v-if="item.icon" :class="item.icon" />
        </slot>
        <span>{{ item.label }}</span>
      </a>
    </template>
  </ContextMenu>
</template>

<style>
/* Light theme */
.mtchat-contextmenu--light {
  --p-contextmenu-background: var(--p-surface-0, #ffffff);
  --p-contextmenu-color: var(--p-text-color, #3f3f46);
  --p-contextmenu-border-color: var(--p-surface-200, #e4e4e7);
  --p-contextmenu-item-color: var(--p-text-color, #3f3f46);
  --p-contextmenu-item-focus-background: var(--p-surface-100, #f4f4f5);
  --p-contextmenu-item-focus-color: var(--p-text-color, #3f3f46);
}

/* Dark theme */
.mtchat-contextmenu--dark {
  --p-contextmenu-background: var(--p-surface-800, #27272a);
  --p-contextmenu-color: var(--p-surface-0, #fafafa);
  --p-contextmenu-border-color: var(--p-surface-700, #3f3f46);
  --p-contextmenu-item-color: var(--p-surface-0, #fafafa);
  --p-contextmenu-item-focus-background: var(--p-surface-700, #3f3f46);
  --p-contextmenu-item-focus-color: var(--p-surface-0, #fafafa);
}
</style>
