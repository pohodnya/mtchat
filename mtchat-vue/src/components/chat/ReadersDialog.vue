<script setup lang="ts">
/**
 * ReadersDialog - Dialog showing who has read a message
 *
 * Uses registry MtDialog primitive
 */

import type { DialogParticipant } from '../../types'
import { useI18n } from '../../i18n'
import { useRegistry } from '../../registry'

const { t } = useI18n()
const { MtDialog } = useRegistry()

defineProps<{
  show: boolean
  readers: DialogParticipant[]
  theme?: string
}>()

const emit = defineEmits<{
  (e: 'close'): void
}>()

function getInitials(name: string): string {
  if (!name) return '?'
  const parts = name.trim().split(/\s+/)
  if (parts.length >= 2) {
    return (parts[0][0] + parts[1][0]).toUpperCase()
  }
  return name.slice(0, 2).toUpperCase()
}

function handleClose() {
  emit('close')
}
</script>

<template>
  <component
    :is="MtDialog"
    :visible="show"
    :header="t.readReceipts.readBy"
    max-width="320px"
    :theme="theme || 'light'"
    @update:visible="!$event && handleClose()"
    @close="handleClose"
  >
    <div class="readers-dialog__body">
      <div
        v-for="reader in readers"
        :key="reader.user_id"
        class="readers-dialog__item"
      >
        <div class="readers-dialog__avatar">
          {{ getInitials(reader.display_name || '') }}
        </div>
        <div class="readers-dialog__info">
          <span v-if="reader.company" class="readers-dialog__company">{{ reader.company }}</span>
          <span v-if="reader.company && reader.display_name" class="readers-dialog__separator"> — </span>
          <span class="readers-dialog__name">{{ reader.display_name || t.user.defaultName }}</span>
        </div>
      </div>
      <div v-if="readers.length === 0" class="readers-dialog__empty">
        {{ t.chat.noMessages }}
      </div>
    </div>
  </component>
</template>

<style>
.readers-dialog__body {
  max-height: 300px;
  overflow-y: auto;
}

.readers-dialog__item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px;
  border-radius: 6px;
  transition: background-color 0.15s;
}

.readers-dialog__item:hover {
  background: var(--mtchat-hover);
}

.readers-dialog__avatar {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  background: var(--mtchat-primary);
  color: white;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 12px;
  font-weight: 600;
  flex-shrink: 0;
}

.readers-dialog__info {
  font-size: 13px;
  line-height: 1.4;
  min-width: 0;
}

.readers-dialog__company {
  color: var(--mtchat-text-secondary);
}

.readers-dialog__separator {
  color: var(--mtchat-text-secondary);
}

.readers-dialog__name {
  color: var(--mtchat-text);
}

.readers-dialog__empty {
  padding: 24px;
  text-align: center;
  color: var(--mtchat-text-secondary);
  font-size: 13px;
}
</style>
