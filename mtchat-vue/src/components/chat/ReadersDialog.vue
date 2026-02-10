<template>
  <Teleport to="body">
    <Transition name="readers-dialog">
      <div v-if="show" :class="['readers-dialog-overlay', `readers-dialog--${theme || 'light'}`]" @click.self="$emit('close')">
        <div class="readers-dialog">
          <div class="readers-dialog__header">
            <h2 class="readers-dialog__title">{{ t.readReceipts.readBy }}</h2>
            <button
              class="readers-dialog__close"
              @click="$emit('close')"
              :title="t.tooltips.close"
            >
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <line x1="18" y1="6" x2="6" y2="18"/>
                <line x1="6" y1="6" x2="18" y2="18"/>
              </svg>
            </button>
          </div>

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
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { watch, onUnmounted } from 'vue'
import type { DialogParticipant } from '../../types'
import { useI18n } from '../../i18n'

const { t } = useI18n()

const props = defineProps<{
  show: boolean
  readers: DialogParticipant[]
  theme?: 'light' | 'dark'
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

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape' && props.show) {
    emit('close')
  }
}

watch(() => props.show, (show) => {
  if (show) {
    document.addEventListener('keydown', handleKeydown)
  } else {
    document.removeEventListener('keydown', handleKeydown)
  }
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeydown)
})
</script>

<style scoped>
.readers-dialog-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

/* Light theme */
.readers-dialog--light {
  --mtchat-bg: #ffffff;
  --mtchat-text: #334155;
  --mtchat-text-secondary: #64748b;
  --mtchat-border: #e2e8f0;
  --mtchat-hover: #f1f5f9;
  --mtchat-primary: #3B82F6;
}

/* Dark theme */
.readers-dialog--dark {
  --mtchat-bg: #1f2937;
  --mtchat-text: #f8fafc;
  --mtchat-text-secondary: #94a3b8;
  --mtchat-border: #374151;
  --mtchat-hover: #374151;
  --mtchat-primary: #60a5fa;
}

.readers-dialog {
  background: var(--mtchat-bg);
  border-radius: 12px;
  width: 100%;
  max-width: 320px;
  max-height: 400px;
  margin: 16px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.2);
  color: var(--mtchat-text);
  display: flex;
  flex-direction: column;
}

.readers-dialog__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 16px;
  border-bottom: 1px solid var(--mtchat-border);
  flex-shrink: 0;
}

.readers-dialog__title {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
}

.readers-dialog__close {
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  background: none;
  border-radius: 6px;
  cursor: pointer;
  color: var(--mtchat-text-secondary);
  transition: background-color 0.2s, color 0.2s;
}

.readers-dialog__close:hover {
  background: var(--mtchat-hover);
  color: var(--mtchat-text);
}

.readers-dialog__body {
  padding: 8px;
  overflow-y: auto;
  flex: 1;
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

/* Transition */
.readers-dialog-enter-active,
.readers-dialog-leave-active {
  transition: opacity 0.2s ease;
}

.readers-dialog-enter-active .readers-dialog,
.readers-dialog-leave-active .readers-dialog {
  transition: transform 0.2s ease, opacity 0.2s ease;
}

.readers-dialog-enter-from,
.readers-dialog-leave-to {
  opacity: 0;
}

.readers-dialog-enter-from .readers-dialog,
.readers-dialog-leave-to .readers-dialog {
  transform: scale(0.95);
  opacity: 0;
}
</style>
