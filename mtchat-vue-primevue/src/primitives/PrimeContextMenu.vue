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
