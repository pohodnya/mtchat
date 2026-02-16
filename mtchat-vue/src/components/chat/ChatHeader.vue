<script setup lang="ts">
/**
 * ChatHeader - Dialog header with title, status, and actions menu
 */

import { ref, computed } from 'vue'
import type { DialogListItem } from '../../types'
import { useI18n } from '../../i18n'
import Icon from '../Icon.vue'

const props = defineProps<{
  dialog: DialogListItem
  isConnected: boolean
  isLoading: boolean
  showBackButton?: boolean
  isInlineMode?: boolean
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

// Slots for custom menu actions
defineSlots<{
  'menu-actions'(props: { closeMenu: () => void; menuItemClass: string }): any
}>()

// i18n
const { t, tt } = useI18n()

// State
const showMenu = ref(false)

// Computed title
const dialogTitle = computed(() => props.dialog.title || `${props.dialog.object_type}/${props.dialog.object_id}`)

function closeMenu() {
  showMenu.value = false
}
</script>

<template>
  <header class="chat-header">
    <!-- Back button -->
    <button
      v-if="showBackButton"
      class="chat-header__back"
      :title="t.tooltips.back"
      @click="emit('back')"
    >
      <Icon name="chevron-left" :size="20" />
    </button>

    <!-- Dialog info (clickable) -->
    <button
      class="chat-header__info"
      :title="t.tooltips.chatInfo"
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
        <span :class="['chat-header__status', { 'chat-header__status--connected': isConnected }]">
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
          @click="showMenu = !showMenu"
        >
          <Icon name="more-vertical" :size="20" />
        </button>

        <div v-if="showMenu" class="chat-header__menu" @click.stop>
          <button class="chat-header__menu-item" @click="emit('showInfo'); closeMenu()">
            <Icon name="info" :size="16" />
            {{ t.buttons.info }}
          </button>
          <button
            class="chat-header__menu-item"
            :disabled="isLoading"
            @click="emit('toggleNotifications'); closeMenu()"
          >
            <Icon v-if="dialog.notifications_enabled !== false" name="bell" :size="16" />
            <Icon v-else name="bell-off" :size="16" />
            {{ dialog.notifications_enabled !== false ? t.buttons.muteNotifications : t.buttons.unmuteNotifications }}
          </button>
          <button
            v-if="!dialog.is_archived"
            class="chat-header__menu-item"
            :disabled="isLoading"
            @click="dialog.is_pinned ? emit('unpin') : emit('pin'); closeMenu()"
          >
            <Icon v-if="dialog.is_pinned" name="unpin" :size="16" />
            <Icon v-else name="pin" :size="16" />
            {{ dialog.is_pinned ? t.buttons.unpin : t.buttons.pin }}
          </button>
          <button
            class="chat-header__menu-item"
            :disabled="isLoading"
            @click="dialog.is_archived ? emit('unarchive') : emit('archive'); closeMenu()"
          >
            <Icon name="archive" :size="16" />
            {{ dialog.is_archived ? t.buttons.unarchive : t.buttons.archive }}
          </button>

          <!-- Custom actions slot -->
          <slot name="menu-actions" :close-menu="closeMenu" :menu-item-class="'chat-header__menu-item'" />

          <button
            class="chat-header__menu-item chat-header__menu-item--danger"
            :disabled="isLoading"
            @click="emit('leave'); closeMenu()"
          >
            <Icon name="logout" :size="16" />
            {{ t.buttons.leaveChat }}
          </button>
        </div>
      </div>
    </div>

    <!-- Backdrop to close menu -->
    <div v-if="showMenu" class="chat-header__backdrop" @click="closeMenu"></div>
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
  background: #ef4444;
  margin-right: 4px;
}

.chat-header__status--connected::before {
  background: #22c55e;
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

.chat-header__menu {
  position: absolute;
  top: 100%;
  right: 0;
  margin-top: 4px;
  background: var(--mtchat-bg);
  border: 1px solid var(--mtchat-border);
  border-radius: 8px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  z-index: 100;
  min-width: 200px;
  padding: 4px;
}

.chat-header__menu-item {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 10px 12px;
  background: none;
  border: none;
  font-size: 13px;
  color: var(--mtchat-text);
  cursor: pointer;
  border-radius: 6px;
  text-align: left;
}

.chat-header__menu-item:hover {
  background: var(--mtchat-bg-hover);
}

.chat-header__menu-item:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.chat-header__menu-item--danger {
  color: #ef4444;
}

/* Slot content styling */
.chat-header__menu :slotted(.chat-header__menu-item) {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 10px 12px;
  background: none;
  border: none;
  font-size: 13px;
  color: var(--mtchat-text);
  cursor: pointer;
  border-radius: 6px;
  text-align: left;
}

.chat-header__menu :slotted(.chat-header__menu-item:hover) {
  background: var(--mtchat-bg-hover);
}

.chat-header__backdrop {
  position: fixed;
  inset: 0;
  z-index: 50;
}
</style>
