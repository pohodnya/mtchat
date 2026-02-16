<script setup lang="ts">
/**
 * PrimeDialog - PrimeVue Dialog adapter
 */

import { computed } from 'vue'
import Dialog from 'primevue/dialog'
import type { MtDialogProps, MtDialogEmits } from '@mtchat/vue'

const props = withDefaults(defineProps<MtDialogProps>(), {
  modal: true,
  closable: true,
  maxWidth: '400px',
  draggable: false,
})

const emit = defineEmits<MtDialogEmits>()

const dialogPt = computed(() => ({
  root: {
    class: `mtchat-dialog--${props.theme}`,
  },
}))

function handleVisibleChange(value: boolean) {
  emit('update:visible', value)
  if (!value) {
    emit('close')
  }
}
</script>

<template>
  <Dialog
    :visible="visible"
    :header="header"
    :modal="modal"
    :closable="closable"
    :draggable="draggable"
    :style="{ maxWidth, width: '100%' }"
    :pt="dialogPt"
    append-to="self"
    @update:visible="handleVisibleChange"
  >
    <slot />
    <template v-if="$slots.footer" #footer>
      <slot name="footer" />
    </template>
  </Dialog>
</template>

