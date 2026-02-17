<script setup lang="ts">
/**
 * PrimeMenu - PrimeVue Menu adapter
 */

import { ref, computed } from 'vue'
import Menu from 'primevue/menu'
import type { MtMenuProps, MtMenuEmits, MtMenuExpose } from '@mtchat/vue'

const props = withDefaults(defineProps<MtMenuProps>(), {
  popup: true,
})

const emit = defineEmits<MtMenuEmits>()

const menuRef = ref<InstanceType<typeof Menu> | null>(null)

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
    class: props.theme ? `mtchat-menu--${props.theme}` : null,
  },
}))

function toggle(event: Event) {
  menuRef.value?.toggle(event)
}

function show(event: Event) {
  menuRef.value?.show(event)
}

function hide() {
  menuRef.value?.hide()
  emit('hide')
}

defineExpose<MtMenuExpose>({
  toggle,
  show,
  hide,
})
</script>

<template>
  <Menu
    ref="menuRef"
    :model="primeItems"
    :popup="popup"
    :pt="menuPt"
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
  </Menu>
</template>

<style>
/* Light theme */
.mtchat-menu--light {
  --p-menu-background: var(--p-surface-0, #ffffff);
  --p-menu-color: var(--p-text-color, #3f3f46);
  --p-menu-border-color: var(--p-surface-200, #e4e4e7);
  --p-menu-item-color: var(--p-text-color, #3f3f46);
  --p-menu-item-focus-background: var(--p-surface-100, #f4f4f5);
  --p-menu-item-focus-color: var(--p-text-color, #3f3f46);
}

/* Dark theme */
.mtchat-menu--dark {
  --p-menu-background: var(--p-surface-800, #27272a);
  --p-menu-color: var(--p-surface-0, #fafafa);
  --p-menu-border-color: var(--p-surface-700, #3f3f46);
  --p-menu-item-color: var(--p-surface-0, #fafafa);
  --p-menu-item-focus-background: var(--p-surface-700, #3f3f46);
  --p-menu-item-focus-color: var(--p-surface-0, #fafafa);
}
</style>
