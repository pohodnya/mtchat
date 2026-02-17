<script setup lang="ts">
/**
 * ChatHeader - Dialog header with title, status, and actions menu
 */

import { ref, computed } from 'vue'
import type { DialogListItem } from '../../types'
import type { MtMenuItem, MtMenuExpose } from '../../registry/types'
import { useI18n } from '../../i18n'
import { useRegistry } from '../../registry/useRegistry'
import Icon from '../Icon.vue'

// Registry components
const { MtMenu } = useRegistry()

const props = defineProps<{
  dialog: DialogListItem
  isConnected: boolean
  isLoading: boolean
  showBackButton?: boolean
  isInlineMode?: boolean
  theme?: string
}>()

const emit = defineEmits<{
  back: []
  showInfo: []
  leave: []
  archive: []
  unarchive: []
  pin: []
  unpin: []
  toggleNotifications: []
}>()


// i18n
const { t, tt } = useI18n()

// State
const menuRef = ref<MtMenuExpose | null>(null)

// Computed title
const dialogTitle = computed(() => props.dialog.title || `${props.dialog.object_type}/${props.dialog.object_id}`)

// Menu items
const menuItems = computed<MtMenuItem[]>(() => {
  const items: MtMenuItem[] = []

  // Info
  items.push({
    label: t.value.buttons.info,
    icon: 'info',
    command: () => emit('showInfo'),
  })

  // Notifications toggle
  items.push({
    label: props.dialog.notifications_enabled !== false
      ? t.value.buttons.muteNotifications
      : t.value.buttons.unmuteNotifications,
    icon: props.dialog.notifications_enabled !== false ? 'bell' : 'bell-off',
    disabled: props.isLoading,
    command: () => emit('toggleNotifications'),
  })

  // Pin/Unpin (not for archived)
  if (!props.dialog.is_archived) {
    items.push({
      label: props.dialog.is_pinned ? t.value.buttons.unpin : t.value.buttons.pin,
      icon: props.dialog.is_pinned ? 'unpin' : 'pin',
      disabled: props.isLoading,
      command: () => props.dialog.is_pinned ? emit('unpin') : emit('pin'),
    })
  }

  // Archive/Unarchive
  items.push({
    label: props.dialog.is_archived ? t.value.buttons.unarchive : t.value.buttons.archive,
    icon: 'archive',
    disabled: props.isLoading,
    command: () => props.dialog.is_archived ? emit('unarchive') : emit('archive'),
  })

  // Separator before leave
  items.push({ label: '', separator: true })

  // Leave chat
  items.push({
    label: t.value.buttons.leaveChat,
    icon: 'logout',
    danger: true,
    disabled: props.isLoading,
    command: () => emit('leave'),
  })

  return items
})

function toggleMenu(event: Event) {
  menuRef.value?.toggle(event)
}
</script>

<template>
  <header class="chat-header">
    <!-- Back button -->
    <button
      v-if="showBackButton"
      class="chat-header__back"
      :title="t.tooltips.back"
      :aria-label="t.tooltips.back"
      @click="emit('back')"
    >
      <Icon name="chevron-left" :size="20" />
    </button>

    <!-- Dialog info (clickable) -->
    <button
      class="chat-header__info"
      :title="t.tooltips.chatInfo"
      :aria-label="t.tooltips.chatInfo"
      @click="emit('showInfo')"
    >
      <div class="chat-header__title-row">
        <h2 class="chat-header__title">{{ dialogTitle }}</h2>
        <a
          v-if="!isInlineMode && dialog.object_url"
          :href="dialog.object_url"
          target="_blank"
          rel="noopener noreferrer"
          class="chat-header__link"
          :title="t.tooltips.openObject"
          @click.stop
        >
          <Icon name="external-link" :size="14" />
        </a>
        <span v-if="dialog.is_archived" class="chat-header__badge">
          {{ t.chat.archived }}
        </span>
      </div>
      <div class="chat-header__meta">
        <span class="chat-header__participants">
          {{ tt('chat.participants', { count: dialog.participants_count || 0 }) }}
        </span>
        <span
          role="status"
          :class="['chat-header__status', { 'chat-header__status--connected': isConnected }]"
        >
          {{ isConnected ? t.status.connected : t.status.disconnected }}
        </span>
      </div>
    </button>

    <!-- Actions -->
    <div class="chat-header__actions">
      <!-- Menu (for participants) -->
      <div v-if="dialog.i_am_participant" class="chat-header__menu-container">
        <button
          class="chat-header__menu-btn"
          :title="t.tooltips.menu"
          :aria-label="t.tooltips.menu"
          aria-haspopup="true"
          @click="toggleMenu"
        >
          <Icon name="more-vertical" :size="20" />
        </button>

        <component
          :is="MtMenu"
          ref="menuRef"
          :items="menuItems"
          :popup="true"
          :theme="theme"
        >
          <template #item-icon="{ item }">
            <Icon :name="item.icon" :size="16" />
          </template>
        </component>
      </div>
    </div>
  </header>
</template>

<style scoped>
.chat-header {
  display: flex;
  align-items: center;
  gap: 8px;
  height: var(--mtchat-header-height, 57px);
  padding: 0 12px;
  background: var(--mtchat-bg);
  border-bottom: 1px solid var(--mtchat-border);
  flex-shrink: 0;
}

.chat-header__back {
  padding: 8px;
  background: none;
  border: none;
  color: var(--mtchat-text-secondary);
  cursor: pointer;
  border-radius: 8px;
}

.chat-header__back:hover {
  background: var(--mtchat-bg-hover);
  color: var(--mtchat-text);
}

.chat-header__info {
  flex: 1;
  min-width: 0;
  text-align: left;
  background: none;
  border: none;
  padding: 4px 8px;
  cursor: pointer;
  border-radius: 8px;
  outline: none;
}

.chat-header__title-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.chat-header__title {
  font-size: 15px;
  font-weight: 600;
  color: var(--mtchat-text);
  margin: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.chat-header__link {
  color: var(--mtchat-text-secondary);
  flex-shrink: 0;
}

.chat-header__link:hover {
  color: var(--mtchat-primary);
}

.chat-header__badge {
  padding: 2px 8px;
  background: var(--mtchat-bg-secondary);
  border-radius: 10px;
  font-size: 11px;
  color: var(--mtchat-text-secondary);
  flex-shrink: 0;
}

.chat-header__meta {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 2px;
}

.chat-header__participants {
  font-size: 12px;
  color: var(--mtchat-text-secondary);
}

.chat-header__status {
  font-size: 11px;
  color: var(--mtchat-text-secondary);
}

.chat-header__status::before {
  content: '';
  display: inline-block;
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--mtchat-danger, #ef4444);
  margin-right: 4px;
}

.chat-header__status--connected::before {
  background: var(--mtchat-success, #22c55e);
}

.chat-header__actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.chat-header__menu-container {
  position: relative;
}

.chat-header__menu-btn {
  padding: 8px;
  background: none;
  border: none;
  color: var(--mtchat-text-secondary);
  cursor: pointer;
  border-radius: 8px;
}

.chat-header__menu-btn:hover {
  background: var(--mtchat-bg-hover);
  color: var(--mtchat-text);
}
</style>
