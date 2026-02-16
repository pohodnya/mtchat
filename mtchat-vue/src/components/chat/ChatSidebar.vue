<script setup lang="ts">
/**
 * ChatSidebar - Dialog list with search and tabs
 */

import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import type { DialogListItem } from '../../types'
import type { MtMenuItem, MtContextMenuExpose } from '../../registry/types'
import { useI18n } from '../../i18n'
import { useRegistry } from '../../registry/useRegistry'
import Icon from '../Icon.vue'

// Registry components
const { MtContextMenu } = useRegistry()

const props = defineProps<{
  participatingDialogs: DialogListItem[]
  availableDialogs: DialogListItem[]
  archivedDialogs: DialogListItem[]
  currentDialogId: string | null
  theme: string
}>()

const emit = defineEmits<{
  selectDialog: [dialog: DialogListItem]
  search: [query: string]
  loadArchived: []
  pinDialog: [dialogId: string]
  unpinDialog: [dialogId: string]
  archiveDialog: [dialogId: string]
  unarchiveDialog: [dialogId: string]
  toggleNotifications: [dialogId: string]
}>()

// i18n
const { t, tt } = useI18n()

// Refs
const searchInputRef = ref<HTMLInputElement | null>(null)

// State
const activeTab = ref<'participating' | 'available'>('participating')
const searchInput = ref('')
const showArchivedAccordion = ref(false)
const archivedLoaded = ref(false)
const contextMenuRef = ref<MtContextMenuExpose | null>(null)
const contextMenuDialog = ref<DialogListItem | null>(null)

// Debounce timer
let searchTimeout: ReturnType<typeof setTimeout> | null = null

// Computed
const currentDialogsList = computed(() =>
  activeTab.value === 'participating'
    ? props.participatingDialogs
    : props.availableDialogs
)

const sortedActiveDialogs = computed(() => {
  const dialogs = [...currentDialogsList.value]
  return dialogs.sort((a, b) => {
    // Pinned first
    if (a.is_pinned && !b.is_pinned) return -1
    if (!a.is_pinned && b.is_pinned) return 1
    // Then by last_message_at
    const aTime = a.last_message_at || a.created_at
    const bTime = b.last_message_at || b.created_at
    return new Date(bTime).getTime() - new Date(aTime).getTime()
  })
})

const sortedArchivedDialogs = computed(() => {
  const dialogs = [...props.archivedDialogs]
  return dialogs.sort((a, b) => {
    const aTime = a.last_message_at || a.created_at
    const bTime = b.last_message_at || b.created_at
    return new Date(bTime).getTime() - new Date(aTime).getTime()
  })
})

// Context menu items
const contextMenuItems = computed<MtMenuItem[]>(() => {
  const dialog = contextMenuDialog.value
  if (!dialog || !dialog.i_am_participant) return []

  const items: MtMenuItem[] = []

  // Pin/Unpin (not for archived)
  if (!dialog.is_archived) {
    items.push({
      label: dialog.is_pinned ? t.value.buttons.unpin : t.value.buttons.pin,
      icon: dialog.is_pinned ? 'unpin' : 'pin',
      command: () => {
        if (dialog.is_pinned) {
          emit('unpinDialog', dialog.id)
        } else {
          emit('pinDialog', dialog.id)
        }
      },
    })
  }

  // Notifications toggle
  items.push({
    label: dialog.notifications_enabled !== false
      ? t.value.buttons.muteNotifications
      : t.value.buttons.unmuteNotifications,
    icon: dialog.notifications_enabled !== false ? 'bell' : 'bell-off',
    command: () => {
      emit('toggleNotifications', dialog.id)
    },
  })

  // Archive/Unarchive
  items.push({
    label: dialog.is_archived ? t.value.buttons.unarchive : t.value.buttons.archive,
    icon: 'archive',
    command: () => {
      if (dialog.is_archived) {
        emit('unarchiveDialog', dialog.id)
      } else {
        emit('archiveDialog', dialog.id)
      }
    },
  })

  return items
})

// Debounced search
watch(searchInput, (newValue) => {
  if (searchTimeout) clearTimeout(searchTimeout)
  searchTimeout = setTimeout(() => {
    emit('search', newValue)
  }, 300)
})

function clearSearch() {
  searchInput.value = ''
  emit('search', '')
}

async function toggleArchivedAccordion() {
  showArchivedAccordion.value = !showArchivedAccordion.value
  if (showArchivedAccordion.value && !archivedLoaded.value) {
    emit('loadArchived')
    archivedLoaded.value = true
  }
}

// Context menu
function handleDialogContextMenu(e: MouseEvent, dialog: DialogListItem) {
  // Only show context menu for dialogs where user is participant
  if (!dialog.i_am_participant) return
  contextMenuDialog.value = dialog
  contextMenuRef.value?.show(e)
}

// Keyboard shortcut
function handleKeydown(e: KeyboardEvent) {
  if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
    e.preventDefault()
    searchInputRef.value?.focus()
    searchInputRef.value?.select()
  }
}

onMounted(() => {
  document.addEventListener('keydown', handleKeydown)
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeydown)
  if (searchTimeout) clearTimeout(searchTimeout)
})

// Expose
defineExpose({
  focusSearch: () => {
    searchInputRef.value?.focus()
    searchInputRef.value?.select()
  }
})
</script>

<template>
  <aside class="chat-sidebar">
    <!-- Search -->
    <div class="chat-sidebar__search">
      <div class="chat-sidebar__search-wrapper">
        <input
          ref="searchInputRef"
          v-model="searchInput"
          type="text"
          :placeholder="t.search.placeholder"
          class="chat-sidebar__search-input"
          @keydown.esc="clearSearch"
        />
        <button
          v-if="searchInput"
          class="chat-sidebar__search-clear"
          type="button"
          @click="clearSearch"
        >
          <Icon name="close" :size="14" />
        </button>
      </div>
      <slot name="action"></slot>
    </div>

    <!-- Tabs -->
    <div class="chat-sidebar__tabs">
      <button
        :class="['chat-sidebar__tab', { 'chat-sidebar__tab--active': activeTab === 'participating' }]"
        @click="activeTab = 'participating'"
      >
        {{ t.tabs.myChats }}
        <span v-if="participatingDialogs.length" class="chat-sidebar__tab-count">
          {{ participatingDialogs.length }}
        </span>
      </button>
      <button
        :class="['chat-sidebar__tab', { 'chat-sidebar__tab--active': activeTab === 'available' }]"
        @click="activeTab = 'available'"
      >
        {{ t.tabs.available }}
        <span v-if="availableDialogs.length" class="chat-sidebar__tab-count">
          {{ availableDialogs.length }}
        </span>
      </button>
    </div>

    <!-- Dialog List Container -->
    <div class="chat-sidebar__list-container">
      <!-- Dialog List -->
      <div class="chat-sidebar__list">
        <div
          v-for="dialog in sortedActiveDialogs"
          :key="dialog.id"
          :class="['chat-sidebar__item', { 'chat-sidebar__item--active': currentDialogId === dialog.id }]"
          @click="emit('selectDialog', dialog)"
          @contextmenu="handleDialogContextMenu($event, dialog)"
        >
          <Icon v-if="dialog.is_pinned" name="pin" :size="12" class="chat-sidebar__pin-icon" />
          <Icon v-if="dialog.notifications_enabled === false" name="bell-off" :size="12" class="chat-sidebar__muted-icon" :title="t.tooltips.muted" />
          <div class="chat-sidebar__item-content">
            <div class="chat-sidebar__item-title">
              {{ dialog.title || `${dialog.object_type}/${dialog.object_id}` }}
            </div>
            <div class="chat-sidebar__item-meta">
              {{ tt('chat.participants', { count: dialog.participants_count }) }}
            </div>
          </div>
          <span v-if="dialog.unread_count && dialog.unread_count > 0" class="chat-sidebar__unread">
            {{ dialog.unread_count > 99 ? '99+' : dialog.unread_count }}
          </span>
        </div>

        <div v-if="sortedActiveDialogs.length === 0" class="chat-sidebar__empty">
          {{ searchInput
            ? t.search.noResults
            : (activeTab === 'participating' ? t.chat.noActiveChats : t.chat.noAvailableChats)
          }}
        </div>
      </div>

      <!-- Archived Accordion -->
      <div
        v-if="activeTab === 'participating'"
        :class="['chat-sidebar__archived', { 'chat-sidebar__archived--open': showArchivedAccordion }]"
      >
        <button class="chat-sidebar__archived-toggle" @click="toggleArchivedAccordion">
          <Icon name="chevron-right" :size="12" />
          {{ t.chat.archived }}
        </button>

        <div v-if="showArchivedAccordion" class="chat-sidebar__archived-list">
          <div
            v-for="dialog in sortedArchivedDialogs"
            :key="dialog.id"
            :class="['chat-sidebar__item', 'chat-sidebar__item--archived', { 'chat-sidebar__item--active': currentDialogId === dialog.id }]"
            @click="emit('selectDialog', dialog)"
            @contextmenu="handleDialogContextMenu($event, dialog)"
          >
            <div class="chat-sidebar__item-content">
              <div class="chat-sidebar__item-title">
                {{ dialog.title || `${dialog.object_type}/${dialog.object_id}` }}
              </div>
              <div class="chat-sidebar__item-meta">
                {{ tt('chat.participants', { count: dialog.participants_count }) }}
              </div>
            </div>
            <span v-if="dialog.unread_count && dialog.unread_count > 0" class="chat-sidebar__unread">
              {{ dialog.unread_count > 99 ? '99+' : dialog.unread_count }}
            </span>
          </div>
        </div>
      </div>
    </div>

    <!-- Context Menu -->
    <component
      :is="MtContextMenu"
      ref="contextMenuRef"
      :items="contextMenuItems"
    >
      <template #item-icon="{ item }">
        <Icon :name="item.icon" :size="16" />
      </template>
    </component>
  </aside>
</template>

<style scoped>
.chat-sidebar {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--mtchat-bg);
  border-right: 1px solid var(--mtchat-border);
}

/* Search */
.chat-sidebar__search {
  display: flex;
  align-items: center;
  gap: 8px;
  height: var(--mtchat-header-height, 57px);
  padding: 0 12px;
  border-bottom: 1px solid var(--mtchat-border);
  flex-shrink: 0;
}

.chat-sidebar__search-wrapper {
  flex: 1;
  position: relative;
}

.chat-sidebar__search-input {
  width: 100%;
  padding: 8px 32px 8px 12px;
  border: 1px solid var(--mtchat-border);
  border-radius: 8px;
  background: var(--mtchat-bg-secondary);
  color: var(--mtchat-text);
  font-size: 14px;
}

.chat-sidebar__search-input:focus {
  outline: none;
  border-color: var(--mtchat-primary);
}

.chat-sidebar__search-clear {
  position: absolute;
  right: 8px;
  top: 50%;
  transform: translateY(-50%);
  padding: 4px;
  background: none;
  border: none;
  cursor: pointer;
  color: var(--mtchat-text-secondary);
}

/* Tabs */
.chat-sidebar__tabs {
  display: flex;
  border-bottom: 1px solid var(--mtchat-border);
}

.chat-sidebar__tab {
  flex: 1;
  padding: 12px;
  background: none;
  border: none;
  border-bottom: 2px solid transparent;
  font-size: 14px;
  font-weight: 500;
  color: var(--mtchat-text-secondary);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
}

.chat-sidebar__tab:hover {
  background: var(--mtchat-bg-hover);
}

.chat-sidebar__tab--active {
  color: var(--mtchat-primary);
  border-bottom-color: var(--mtchat-primary);
}

.chat-sidebar__tab-count {
  padding: 2px 6px;
  background: var(--mtchat-bg-secondary);
  border-radius: 10px;
  font-size: 11px;
}

/* List container */
.chat-sidebar__list-container {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.chat-sidebar__list {
  flex: 1;
  overflow-y: auto;
}

/* Dialog item */
.chat-sidebar__item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px;
  cursor: pointer;
  border-bottom: 1px solid var(--mtchat-border);
}

.chat-sidebar__item:hover {
  background: var(--mtchat-bg-hover);
}

.chat-sidebar__item--active {
  background: var(--mtchat-bg-secondary);
}

.chat-sidebar__item--archived {
  opacity: 0.7;
}

.chat-sidebar__pin-icon {
  color: var(--mtchat-primary);
  flex-shrink: 0;
}

.chat-sidebar__muted-icon {
  color: var(--mtchat-text-secondary);
  flex-shrink: 0;
}

.chat-sidebar__item-content {
  flex: 1;
  min-width: 0;
}

.chat-sidebar__item-title {
  font-size: 14px;
  font-weight: 500;
  color: var(--mtchat-text);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.chat-sidebar__item-meta {
  font-size: 12px;
  color: var(--mtchat-text-secondary);
  margin-top: 2px;
}

.chat-sidebar__unread {
  padding: 2px 8px;
  background: var(--mtchat-primary);
  color: white;
  border-radius: 10px;
  font-size: 11px;
  font-weight: 600;
  flex-shrink: 0;
}

.chat-sidebar__empty {
  padding: 24px;
  text-align: center;
  color: var(--mtchat-text-secondary);
  font-size: 14px;
}

/* Archived section */
.chat-sidebar__archived {
  border-top: 1px solid var(--mtchat-border);
  max-height: 50%;
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
}

.chat-sidebar__archived-toggle {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 12px;
  background: var(--mtchat-bg-secondary);
  border: none;
  font-size: 13px;
  font-weight: 500;
  color: var(--mtchat-text-secondary);
  cursor: pointer;
}

.chat-sidebar__archived-toggle:hover {
  background: var(--mtchat-bg-hover);
}

.chat-sidebar__archived--open .chat-sidebar__archived-toggle svg {
  transform: rotate(90deg);
}

.chat-sidebar__archived-list {
  flex: 1;
  overflow-y: auto;
  min-height: 0;
}

</style>
