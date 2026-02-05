<template>
  <div class="chat-info-panel">
    <!-- Header -->
    <div class="chat-info-panel__header">
      <h2 class="chat-info-panel__title">{{ t.infoPanel.title }}</h2>
      <button
        class="chat-info-panel__close"
        @click="$emit('close')"
        :title="t.tooltips.close"
      >
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <line x1="18" y1="6" x2="6" y2="18"/>
          <line x1="6" y1="6" x2="18" y2="18"/>
        </svg>
      </button>
    </div>

    <!-- Dialog info -->
    <div class="chat-info-panel__section">
      <h3 class="chat-info-panel__section-title">{{ dialogTitle }}</h3>
      <div v-if="objectType" class="chat-info-panel__object-info">
        <span class="chat-info-panel__object-type">{{ objectTypeLabel }}</span>
        <span v-if="objectId" class="chat-info-panel__object-id">#{{ objectId.slice(0, 8) }}</span>
      </div>
    </div>

    <!-- Participants -->
    <div class="chat-info-panel__section">
      <h3 class="chat-info-panel__section-title">
        {{ t.infoPanel.participants }} ({{ participants.length }})
      </h3>
      <div class="chat-info-panel__participants">
        <div
          v-for="participant in sortedParticipants"
          :key="participant.user_id"
          class="chat-info-panel__participant"
        >
          <!-- Avatar -->
          <div class="chat-info-panel__avatar">
            {{ getInitials(participant.display_name || 'U') }}
          </div>

          <!-- Info -->
          <div class="chat-info-panel__participant-info">
            <div class="chat-info-panel__participant-name">
              {{ participant.display_name || t.user.defaultName }}
              <span v-if="participant.user_id === currentUserId" class="chat-info-panel__you-badge">{{ t.user.youBadge }}</span>
              <span v-if="participant.joined_as === 'creator'" class="chat-info-panel__creator-badge">{{ t.user.creator }}</span>
            </div>

            <div v-if="participant.company" class="chat-info-panel__participant-company">
              {{ participant.company }}
            </div>

            <div class="chat-info-panel__participant-contacts">
              <a
                v-if="participant.email"
                :href="`mailto:${participant.email}`"
                class="chat-info-panel__contact"
              >
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M4 4h16c1.1 0 2 .9 2 2v12c0 1.1-.9 2-2 2H4c-1.1 0-2-.9-2-2V6c0-1.1.9-2 2-2z"/>
                  <polyline points="22,6 12,13 2,6"/>
                </svg>
                {{ participant.email }}
              </a>
              <a
                v-if="participant.phone"
                :href="`tel:${participant.phone}`"
                class="chat-info-panel__contact"
              >
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M22 16.92v3a2 2 0 01-2.18 2 19.79 19.79 0 01-8.63-3.07 19.5 19.5 0 01-6-6 19.79 19.79 0 01-3.07-8.67A2 2 0 014.11 2h3a2 2 0 012 1.72 12.84 12.84 0 00.7 2.81 2 2 0 01-.45 2.11L8.09 9.91a16 16 0 006 6l1.27-1.27a2 2 0 012.11-.45 12.84 12.84 0 002.81.7A2 2 0 0122 16.92z"/>
                </svg>
                {{ participant.phone }}
              </a>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted } from 'vue'
import type { DialogParticipant } from '../../types'
import { useI18n } from '../../i18n'

const { t } = useI18n()

const props = defineProps<{
  dialogTitle: string
  objectType?: string
  objectId?: string
  participants: DialogParticipant[]
  currentUserId: string
}>()

const emit = defineEmits<{
  (e: 'close'): void
}>()

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    emit('close')
  }
}

onMounted(() => {
  document.addEventListener('keydown', handleKeydown)
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeydown)
})

const objectTypeLabel = computed(() => {
  const labels = t.value.infoPanel.objectTypes
  const typeMap: Record<string, string> = {
    tender: labels.tender,
    order: labels.order,
    route: labels.route,
  }
  return props.objectType ? typeMap[props.objectType] || props.objectType : ''
})

// Sort participants: current user first, then by join date
const sortedParticipants = computed(() => {
  return [...props.participants].sort((a, b) => {
    // Current user first
    if (a.user_id === props.currentUserId) return -1
    if (b.user_id === props.currentUserId) return 1
    // Creator next
    if (a.joined_as === 'creator') return -1
    if (b.joined_as === 'creator') return 1
    // Then by join date
    return new Date(a.joined_at).getTime() - new Date(b.joined_at).getTime()
  })
})

function getInitials(name: string): string {
  const parts = name.trim().split(/\s+/)
  if (parts.length >= 2) {
    return (parts[0][0] + parts[1][0]).toUpperCase()
  }
  return name.slice(0, 2).toUpperCase()
}
</script>

<style scoped>
.chat-info-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--mtchat-bg, #ffffff);
  color: var(--mtchat-text, #1a1a1a);
}

.chat-info-panel__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px;
  border-bottom: 1px solid var(--mtchat-border, #e5e5e5);
}

.chat-info-panel__title {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
}

.chat-info-panel__close {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  background: none;
  border-radius: 6px;
  cursor: pointer;
  color: var(--mtchat-text-secondary, #666);
  transition: background-color 0.2s, color 0.2s;
}

.chat-info-panel__close:hover {
  background: var(--mtchat-bg-hover, rgba(0, 0, 0, 0.05));
  color: var(--mtchat-text, #1a1a1a);
}

.chat-info-panel__section {
  padding: 16px;
  border-bottom: 1px solid var(--mtchat-border, #e5e5e5);
}

.chat-info-panel__section:last-child {
  border-bottom: none;
  flex: 1;
  overflow-y: auto;
}

.chat-info-panel__section-title {
  margin: 0 0 12px;
  font-size: 14px;
  font-weight: 600;
  color: var(--mtchat-text, #1a1a1a);
}

.chat-info-panel__object-info {
  display: flex;
  gap: 8px;
  font-size: 13px;
  color: var(--mtchat-text-secondary, #666);
}

.chat-info-panel__participants {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.chat-info-panel__participant {
  display: flex;
  gap: 12px;
  padding: 8px;
  border-radius: 8px;
  transition: background-color 0.2s;
}

.chat-info-panel__participant:hover {
  background: var(--mtchat-bg-hover, rgba(0, 0, 0, 0.05));
}

.chat-info-panel__avatar {
  width: 40px;
  height: 40px;
  border-radius: 50%;
  background: var(--mtchat-primary, #007AFF);
  color: white;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 14px;
  font-weight: 600;
  flex-shrink: 0;
}

.chat-info-panel__participant-info {
  flex: 1;
  min-width: 0;
}

.chat-info-panel__participant-name {
  font-weight: 500;
  font-size: 14px;
  display: flex;
  align-items: center;
  gap: 6px;
  flex-wrap: wrap;
}

.chat-info-panel__you-badge {
  font-size: 12px;
  color: var(--mtchat-text-secondary, #666);
  font-weight: normal;
}

.chat-info-panel__creator-badge {
  font-size: 11px;
  padding: 1px 6px;
  background: var(--mtchat-primary, #007AFF);
  color: white;
  border-radius: 4px;
  font-weight: 500;
}

.chat-info-panel__participant-company {
  font-size: 13px;
  color: var(--mtchat-text-secondary, #666);
  margin-top: 2px;
}

.chat-info-panel__participant-contacts {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-top: 4px;
}

.chat-info-panel__contact {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  color: var(--mtchat-primary, #007AFF);
  text-decoration: none;
}

.chat-info-panel__contact:hover {
  text-decoration: underline;
}

.chat-info-panel__contact svg {
  flex-shrink: 0;
}
</style>
