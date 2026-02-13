<script setup lang="ts">
/**
 * MTChat - Main chat component
 *
 * Two display modes:
 * - Full mode: Dialog list (sidebar) + chat area
 * - Inline mode: Single chat for a business object
 */

import { computed, ref, watch, nextTick, onMounted, onUnmounted } from 'vue'
import { useChat } from '../composables/useChat'
import { provideI18n } from '../i18n'
import type { MTChatConfig, ChatMode, DialogListItem, Message, Attachment } from '../types'

// Sub-components
import ChatSidebar from './chat/ChatSidebar.vue'
import ChatHeader from './chat/ChatHeader.vue'
import ChatMessages from './chat/ChatMessages.vue'
import ChatInput from './chat/ChatInput.vue'
import ChatInfoPanel from './chat/ChatInfoPanel.vue'
import JoinDialog from './chat/JoinDialog.vue'
import FileViewer from './chat/FileViewer.vue'
import ReadersDialog from './chat/ReadersDialog.vue'
import Icon from './Icon.vue'

// Props
const props = withDefaults(
  defineProps<{
    config: MTChatConfig
    mode?: ChatMode
    objectId?: string
    objectType?: string
    dialogId?: string
    showHeader?: boolean
    showSidebar?: boolean
    theme?: 'light' | 'dark'
  }>(),
  {
    mode: 'full',
    showHeader: true,
    showSidebar: true,
    theme: 'light',
  }
)

// Emits
const emit = defineEmits<{
  connected: []
  disconnected: []
  error: [error: Error]
  'message-sent': [message: Message]
  'dialog-selected': [dialog: DialogListItem]
  'dialog-joined': [dialogId: string]
  'dialog-left': [dialogId: string]
}>()

// i18n
const { t, localeRef } = provideI18n(props.config.locale ?? 'ru')

watch(() => props.config.locale, (newLocale) => {
  if (newLocale && newLocale !== localeRef.value) {
    localeRef.value = newLocale
  }
})

// Chat composable
const chat = useChat({
  config: props.config,
  dialogId: props.dialogId,
  objectId: props.objectId,
  objectType: props.objectType,
})

// Refs
const containerRef = ref<HTMLElement | null>(null)
const messagesRef = ref<InstanceType<typeof ChatMessages> | null>(null)
const inputRef = ref<InstanceType<typeof ChatInput> | null>(null)

// UI State
const showInfoPanel = ref(false)
const showJoinDialog = ref(false)
const isJoining = ref(false)
const showFileViewer = ref(false)
const fileViewerIndex = ref(0)
const readersDialogMessage = ref<Message | null>(null)

// Responsive state
const windowWidth = ref(typeof window !== 'undefined' ? window.innerWidth : 1200)
const isMobile = computed(() => windowWidth.value < 768)
const isTablet = computed(() => windowWidth.value >= 768 && windowWidth.value < 1200)
const isDesktop = computed(() => windowWidth.value >= 1200)

// Mobile view state
type MobileView = 'list' | 'chat' | 'info'
const mobileView = ref<MobileView>('list')

// Column resize state (desktop)
const sidebarWidth = ref(280)
const infoWidth = ref(300)
const isResizingSidebar = ref(false)
const isResizingInfo = ref(false)

// Resize constraints
const SIDEBAR_MIN_WIDTH = 200
const INFO_MIN_WIDTH = 240
const MAX_COLUMN_PERCENT = 30
const MAIN_MIN_PERCENT = 50

// Computed
const isInlineMode = computed(() => props.mode === 'inline')
const hasDialog = computed(() => chat.currentDialog.value !== null)
const canSendMessage = computed(() => !!(hasDialog.value && chat.currentDialog.value?.i_am_participant))
const canJoin = computed(() => !!(chat.currentDialog.value?.can_join && !chat.currentDialog.value?.i_am_participant))

const dialogTitle = computed(() => {
  const dialog = chat.currentDialog.value
  if (!dialog) return ''
  return dialog.title || `${dialog.object_type}/${dialog.object_id}`
})

// All attachments for file viewer
const allAttachments = computed(() => {
  const attachments: Attachment[] = []
  for (const message of chat.messages.value) {
    if (message.attachments) {
      attachments.push(...message.attachments)
    }
  }
  return attachments
})

// ============ Watchers ============

watch(() => chat.isConnected.value, (connected) => {
  if (connected) {
    emit('connected')
    if (!isInlineMode.value) {
      chat.loadParticipatingDialogs()
      chat.loadAvailableDialogs()
    }
  } else {
    emit('disconnected')
  }
})

watch(() => chat.error.value, (error) => {
  if (error) emit('error', error)
})

watch(showInfoPanel, (show) => {
  if (isMobile.value || isTablet.value) {
    if (show) mobileView.value = 'info'
    else if (mobileView.value === 'info') mobileView.value = 'chat'
  }
})

watch(isDesktop, (desktop) => {
  if (desktop) mobileView.value = 'list'
})

// Auto-fetch missing reply messages
watch(() => chat.messages.value, (messages) => {
  for (const msg of messages) {
    if (msg.reply_to_id) {
      const replyMsg = chat.getReplyMessage(msg.reply_to_id)
      if (replyMsg === undefined) {
        chat.fetchReplyMessage(msg.reply_to_id)
      }
    }
  }
}, { immediate: true })

// ============ Handlers ============

async function handleSelectDialog(dialog: DialogListItem) {
  await chat.selectDialog(dialog.id)
  emit('dialog-selected', dialog)
  if (isMobile.value || isTablet.value) {
    mobileView.value = 'chat'
  }
}

function handleSidebarSearch(query: string) {
  chat.setSearchQuery(query)
  chat.loadParticipatingDialogs()
  chat.loadAvailableDialogs()
}

function handleJoin() {
  showJoinDialog.value = true
}

async function confirmJoin(profile: { display_name: string; company: string; email?: string; phone?: string }) {
  if (!chat.currentDialog.value) return
  const dialogId = chat.currentDialog.value.id

  try {
    isJoining.value = true
    await chat.joinDialog(dialogId, profile)
    showJoinDialog.value = false
    await chat.selectDialog(dialogId)
    emit('dialog-joined', dialogId)
  } finally {
    isJoining.value = false
  }
}

async function handleLeave() {
  if (!chat.currentDialog.value) return
  const dialogId = chat.currentDialog.value.id
  await chat.leaveDialog(dialogId)
  if (!isInlineMode.value) {
    await chat.loadAvailableDialogs()
  }
  emit('dialog-left', dialogId)
}

async function handleToggleNotifications() {
  if (!chat.currentDialog.value) return
  await chat.toggleNotifications(chat.currentDialog.value.id)
}

// Messages handlers
function handleLoadOlder() {
  chat.loadOlderMessages()
}

function handleLoadNewer() {
  chat.loadNewerMessages()
}

function handleScrollToBottom() {
  chat.enableScrollCooldown()
}

async function handleResetToLatest() {
  await chat.resetToLatest()
  await nextTick()
  messagesRef.value?.scrollToBottom(true)
}

function handleReply(message: Message) {
  chat.setReplyTo(message)
}

function handleEdit(message: Message) {
  chat.setEditMessage(message)
  nextTick(() => {
    inputRef.value?.setContent(message.content)
    inputRef.value?.focus()
  })
}

function handleMarkAsRead() {
  chat.markAsRead()
}

async function handleJumpToMessage(messageId: string) {
  const found = await chat.jumpToMessage(messageId)
  console.log('[MTChat] jumpToMessage result:', { found, messageId })
  if (found) {
    await nextTick()
    console.log('[MTChat] calling scrollToMessage, ref exists:', !!messagesRef.value)
    messagesRef.value?.scrollToMessage(messageId)
  }
}

// Input handlers
async function handleSend(content: string, attachments?: any[]) {
  const message = await chat.sendMessage(content, attachments)
  if (message) {
    emit('message-sent', message)
  }
}

async function handleEditSubmit(messageId: string, content: string) {
  await chat.editMessage(messageId, content)
}

// File viewer
function handleOpenGallery(attachment: Attachment) {
  const index = allAttachments.value.findIndex(a => a.id === attachment.id)
  if (index !== -1) {
    fileViewerIndex.value = index
    showFileViewer.value = true
  }
}

function handleOpenFile(attachment: Attachment) {
  handleOpenGallery(attachment)
}

function getCurrentMessageReaders() {
  if (!readersDialogMessage.value) return []
  const messageSentAt = new Date(readersDialogMessage.value.sent_at).getTime()
  const messageTimestamps = new Map<string, number>()
  for (const m of chat.messages.value) {
    messageTimestamps.set(m.id, new Date(m.sent_at).getTime())
  }

  return chat.participants.value.filter(p => {
    if (p.user_id === readersDialogMessage.value?.sender_id) return false
    if (!p.last_read_message_id) return false
    const lastReadTimestamp = messageTimestamps.get(p.last_read_message_id)
    if (!lastReadTimestamp) return false
    return lastReadTimestamp >= messageSentAt
  })
}

// Navigation
function goBack() {
  if (isInlineMode.value && showInfoPanel.value) {
    showInfoPanel.value = false
    return
  }
  if (mobileView.value === 'info') {
    mobileView.value = 'chat'
    showInfoPanel.value = false
  } else if (mobileView.value === 'chat') {
    mobileView.value = 'list'
  }
}

// ============ Resize Handlers ============

function startSidebarResize(e: MouseEvent | TouchEvent) {
  if (!isDesktop.value) return
  e.preventDefault()
  isResizingSidebar.value = true
  document.addEventListener('mousemove', handleSidebarResize)
  document.addEventListener('mouseup', stopResize)
  document.addEventListener('touchmove', handleSidebarResize)
  document.addEventListener('touchend', stopResize)
}

function startInfoResize(e: MouseEvent | TouchEvent) {
  if (!isDesktop.value) return
  e.preventDefault()
  isResizingInfo.value = true
  document.addEventListener('mousemove', handleInfoResize)
  document.addEventListener('mouseup', stopResize)
  document.addEventListener('touchmove', handleInfoResize)
  document.addEventListener('touchend', stopResize)
}

function handleSidebarResize(e: MouseEvent | TouchEvent) {
  if (!isResizingSidebar.value || !containerRef.value) return
  const clientX = 'touches' in e ? e.touches[0].clientX : e.clientX
  const containerRect = containerRef.value.getBoundingClientRect()
  const containerWidth = containerRect.width

  let newWidth = clientX - containerRect.left
  const maxWidth = containerWidth * (MAX_COLUMN_PERCENT / 100)
  const minMainWidth = containerWidth * (MAIN_MIN_PERCENT / 100)
  const availableForSidebar = containerWidth - minMainWidth - (showInfoPanel.value ? infoWidth.value : 0)

  newWidth = Math.min(newWidth, availableForSidebar, maxWidth)
  newWidth = Math.max(newWidth, SIDEBAR_MIN_WIDTH)
  sidebarWidth.value = newWidth
}

function handleInfoResize(e: MouseEvent | TouchEvent) {
  if (!isResizingInfo.value || !containerRef.value) return
  const clientX = 'touches' in e ? e.touches[0].clientX : e.clientX
  const containerRect = containerRef.value.getBoundingClientRect()
  const containerWidth = containerRect.width

  let newWidth = containerRect.right - clientX
  const maxWidth = containerWidth * (MAX_COLUMN_PERCENT / 100)
  const minMainWidth = containerWidth * (MAIN_MIN_PERCENT / 100)
  const availableForInfo = containerWidth - minMainWidth - sidebarWidth.value

  newWidth = Math.min(newWidth, availableForInfo, maxWidth)
  newWidth = Math.max(newWidth, INFO_MIN_WIDTH)
  infoWidth.value = newWidth
}

function stopResize() {
  isResizingSidebar.value = false
  isResizingInfo.value = false
  document.removeEventListener('mousemove', handleSidebarResize)
  document.removeEventListener('mouseup', stopResize)
  document.removeEventListener('touchmove', handleSidebarResize)
  document.removeEventListener('touchend', stopResize)
  document.removeEventListener('mousemove', handleInfoResize)
  document.removeEventListener('touchmove', handleInfoResize)
}

// ============ Lifecycle ============

function handleWindowResize() {
  windowWidth.value = window.innerWidth
}

onMounted(() => {
  window.addEventListener('resize', handleWindowResize)
})

onUnmounted(() => {
  window.removeEventListener('resize', handleWindowResize)
  stopResize()
})

// ============ Expose ============

defineExpose({
  client: chat,
  messages: chat.messages,
  participatingDialogs: chat.participatingDialogs,
  availableDialogs: chat.availableDialogs,
  archivedDialogs: chat.archivedDialogs,
  currentDialog: chat.currentDialog,
  isConnected: chat.isConnected,
  selectDialog: chat.selectDialog,
  sendMessage: chat.sendMessage,
  joinDialog: chat.joinDialog,
  leaveDialog: chat.leaveDialog,
  archiveDialog: chat.archiveDialog,
  unarchiveDialog: chat.unarchiveDialog,
  pinDialog: chat.pinDialog,
  unpinDialog: chat.unpinDialog,
  toggleNotifications: chat.toggleNotifications,
})
</script>

<template>
  <div
    ref="containerRef"
    :class="[
      'mtchat',
      `mtchat--${theme}`,
      {
        'mtchat--inline': isInlineMode,
        'mtchat--mobile': isMobile,
        'mtchat--tablet': isTablet,
        'mtchat--desktop': isDesktop,
        'mtchat--view-list': mobileView === 'list',
        'mtchat--view-chat': mobileView === 'chat',
        'mtchat--view-info': mobileView === 'info',
        'mtchat--resizing': isResizingSidebar || isResizingInfo,
      }
    ]"
  >
    <!-- Sidebar -->
    <ChatSidebar
      v-if="!isInlineMode && showSidebar"
      :participating-dialogs="chat.participatingDialogs.value"
      :available-dialogs="chat.availableDialogs.value"
      :archived-dialogs="chat.archivedDialogs.value"
      :current-dialog-id="chat.currentDialog.value?.id ?? null"
      :theme="theme"
      :style="isDesktop ? { width: `${sidebarWidth}px` } : undefined"
      @select-dialog="handleSelectDialog"
      @search="handleSidebarSearch"
      @load-archived="chat.loadArchivedDialogs()"
      @pin-dialog="chat.pinDialog"
      @unpin-dialog="chat.unpinDialog"
      @archive-dialog="chat.archiveDialog"
      @unarchive-dialog="chat.unarchiveDialog"
      @toggle-notifications="chat.toggleNotifications"
    >
      <template #action>
        <slot name="sidebar-action"></slot>
      </template>
    </ChatSidebar>

    <!-- Sidebar Resizer -->
    <div
      v-if="!isInlineMode && showSidebar && isDesktop"
      class="mtchat__resizer"
      @mousedown="startSidebarResize"
      @touchstart="startSidebarResize"
    ></div>

    <!-- Main Chat Area -->
    <main class="mtchat__main">
      <!-- Header -->
      <ChatHeader
        v-if="showHeader && hasDialog && chat.currentDialog.value"
        :dialog="chat.currentDialog.value"
        :is-connected="chat.isConnected.value"
        :is-loading="chat.isLoading.value"
        :show-back-button="(isMobile && mobileView === 'chat') || (isTablet && showInfoPanel) || (isInlineMode && showInfoPanel)"
        :is-inline-mode="isInlineMode"
        @back="goBack"
        @show-info="showInfoPanel = true"
        @join="handleJoin"
        @leave="handleLeave"
        @archive="chat.archiveDialog(chat.currentDialog.value!.id)"
        @unarchive="chat.unarchiveDialog(chat.currentDialog.value!.id)"
        @pin="chat.pinDialog(chat.currentDialog.value!.id)"
        @unpin="chat.unpinDialog(chat.currentDialog.value!.id)"
        @toggle-notifications="handleToggleNotifications"
      >
        <template #menu-actions="{ closeMenu, menuItemClass }">
          <slot
            name="header-menu-actions"
            :dialog="chat.currentDialog.value!"
            :close-menu="closeMenu"
            :menu-item-class="menuItemClass"
          />
        </template>
      </ChatHeader>

      <!-- No Dialog Selected -->
      <div v-if="!hasDialog" class="mtchat__placeholder">
        <p v-if="isInlineMode">{{ t.chat.noChatForObject }}</p>
        <p v-else>{{ t.chat.selectChat }}</p>
      </div>

      <!-- Join Required -->
      <div v-else-if="!chat.currentDialog.value?.i_am_participant" class="mtchat__join-required">
        <div class="mtchat__join-required-content">
          <Icon name="lock" :size="48" />
          <p>{{ t.chat.joinRequired }}</p>
          <button
            v-if="canJoin"
            class="mtchat__btn mtchat__btn--primary"
            :disabled="chat.isLoading.value"
            @click="handleJoin"
          >
            {{ t.buttons.join }}
          </button>
        </div>
      </div>

      <!-- Messages -->
      <ChatMessages
        v-else
        ref="messagesRef"
        :messages="chat.messages.value"
        :participants="chat.participants.value"
        :current-user-id="config.userId"
        :first-unread-message-id="chat.firstUnreadMessageId.value"
        :is-loading-older="chat.isLoadingOlder.value"
        :is-loading-newer="chat.isLoadingNewer.value"
        :has-more-messages="chat.hasMoreMessages.value"
        :has-more-after="chat.hasMoreAfter.value"
        :is-jumping-to-message="chat.isJumpingToMessage.value"
        :jump-cooldown="chat.jumpCooldown.value"
        :reply-messages-cache="chat.replyMessagesCache.value"
        @load-older="handleLoadOlder"
        @load-newer="handleLoadNewer"
        @scroll-to-bottom="handleScrollToBottom"
        @reset-to-latest="handleResetToLatest"
        @reply="handleReply"
        @edit="handleEdit"
        @open-gallery="handleOpenGallery"
        @open-file="handleOpenFile"
        @mark-as-read="handleMarkAsRead"
        @jump-to-message="handleJumpToMessage"
      />

      <!-- Input -->
      <ChatInput
        v-if="hasDialog"
        ref="inputRef"
        :dialog-id="chat.currentDialog.value!.id"
        :api="chat.api"
        :participants="chat.participants.value"
        :current-user-id="config.userId"
        :reply-to-message="chat.replyToMessage.value"
        :editing-message="chat.editingMessage.value"
        :is-loading="chat.isLoading.value"
        :can-send="canSendMessage"
        :can-join="canJoin"
        @send="handleSend"
        @edit="handleEditSubmit"
        @cancel-reply="chat.clearReplyTo()"
        @cancel-edit="chat.clearEditMessage()"
        @join="handleJoin"
      />
    </main>

    <!-- Info Panel Resizer -->
    <div
      v-if="showInfoPanel && hasDialog && isDesktop && !isInlineMode"
      class="mtchat__resizer"
      @mousedown="startInfoResize"
      @touchstart="startInfoResize"
    ></div>

    <!-- Info Panel -->
    <Transition name="mtchat-info-panel">
      <aside
        v-if="showInfoPanel && hasDialog"
        :class="['mtchat__info-panel', { 'mtchat__info-panel--inline': isInlineMode }]"
        :style="isDesktop && !isInlineMode ? { width: `${infoWidth}px` } : undefined"
      >
        <ChatInfoPanel
          :dialog-title="dialogTitle"
          :object-type="chat.currentDialog.value?.object_type"
          :object-id="chat.currentDialog.value?.object_id"
          :object-url="isInlineMode ? undefined : chat.currentDialog.value?.object_url"
          :participants="chat.participants.value"
          :current-user-id="config.userId"
          @close="showInfoPanel = false"
        />
      </aside>
    </Transition>

    <!-- File Viewer Modal -->
    <FileViewer
      :show="showFileViewer"
      :files="allAttachments"
      :initial-index="fileViewerIndex"
      @close="showFileViewer = false"
    />

    <!-- Join Dialog Modal -->
    <JoinDialog
      :show="showJoinDialog"
      :profile-name="config.userProfile.displayName"
      :company="config.userProfile.company"
      :email="config.userProfile.email"
      :phone="config.userProfile.phone"
      :loading="isJoining"
      :theme="theme"
      @cancel="showJoinDialog = false"
      @join="confirmJoin"
    />

    <!-- Readers Dialog Modal -->
    <ReadersDialog
      :show="readersDialogMessage !== null"
      :readers="getCurrentMessageReaders()"
      :theme="theme"
      @close="readersDialogMessage = null"
    />
  </div>
</template>

<style scoped>
.mtchat {
  /* Layout tokens */
  --mtchat-header-height: 48px;
  --mtchat-resizer-width: 2px;

  /* Spacing tokens */
  --mtchat-spacing-xs: 4px;
  --mtchat-spacing-sm: 8px;
  --mtchat-spacing-md: 12px;
  --mtchat-spacing-lg: 16px;

  display: flex;
  height: 100%;
  background: var(--mtchat-bg);
  color: var(--mtchat-text);
  overflow: hidden;
}

/* Theme colors */
.mtchat--light {
  --mtchat-bg: #ffffff;
  --mtchat-bg-secondary: #f8fafc;
  --mtchat-bg-hover: #f1f5f9;
  --mtchat-text: #1e293b;
  --mtchat-text-secondary: #64748b;
  --mtchat-border: #e2e8f0;
  --mtchat-primary: #3b82f6;
  --mtchat-primary-bg: rgba(59, 130, 246, 0.1);
}

.mtchat--dark {
  --mtchat-bg: #1e1e1e;
  --mtchat-bg-secondary: #2d2d2d;
  --mtchat-bg-hover: #3d3d3d;
  --mtchat-text: #e4e4e7;
  --mtchat-text-secondary: #a1a1aa;
  --mtchat-border: #3f3f46;
  --mtchat-primary: #60a5fa;
  --mtchat-primary-bg: rgba(96, 165, 250, 0.15);
}

/* Main area */
.mtchat__main {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
  min-height: 0;
}

/* Placeholder states */
.mtchat__placeholder,
.mtchat__join-required {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--mtchat-text-secondary);
}

.mtchat__join-required-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
  text-align: center;
  padding: 24px;
}

.mtchat__join-required-content p {
  margin: 0;
}

/* Buttons */
.mtchat__btn {
  padding: 10px 20px;
  border-radius: 8px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  border: none;
}

.mtchat__btn--primary {
  background: var(--mtchat-primary);
  color: white;
}

.mtchat__btn--primary:hover {
  opacity: 0.9;
}

.mtchat__btn--primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* Resizer */
.mtchat__resizer {
  width: var(--mtchat-resizer-width);
  background: var(--mtchat-border);
  cursor: col-resize;
  flex-shrink: 0;
  transition: background-color 0.15s;
}

.mtchat__resizer:hover,
.mtchat--resizing .mtchat__resizer {
  background: var(--mtchat-primary);
}

/* Info panel */
.mtchat__info-panel {
  background: var(--mtchat-bg);
  border-left: 1px solid var(--mtchat-border);
  overflow: hidden;
}

.mtchat__info-panel--inline {
  position: absolute;
  top: 0;
  right: 0;
  bottom: 0;
  width: 100%;
  max-width: 360px;
  z-index: 100;
  box-shadow: -4px 0 12px rgba(0, 0, 0, 0.1);
}

/* Info panel transition */
.mtchat-info-panel-enter-active,
.mtchat-info-panel-leave-active {
  transition: transform 0.2s ease, opacity 0.2s ease;
}

.mtchat-info-panel-enter-from,
.mtchat-info-panel-leave-to {
  transform: translateX(100%);
  opacity: 0;
}

/* Mobile/Tablet responsive */
.mtchat--mobile,
.mtchat--tablet {
  position: relative;
}

.mtchat--mobile .chat-sidebar,
.mtchat--tablet .chat-sidebar {
  position: absolute;
  top: 0;
  left: 0;
  bottom: 0;
  width: 100%;
  z-index: 10;
}

.mtchat--mobile.mtchat--view-chat .chat-sidebar,
.mtchat--mobile.mtchat--view-info .chat-sidebar,
.mtchat--tablet.mtchat--view-chat .chat-sidebar,
.mtchat--tablet.mtchat--view-info .chat-sidebar {
  display: none;
}

.mtchat--mobile.mtchat--view-list .mtchat__main,
.mtchat--tablet.mtchat--view-list .mtchat__main {
  display: none;
}

.mtchat--mobile .mtchat__info-panel,
.mtchat--tablet .mtchat__info-panel {
  position: absolute;
  top: 0;
  right: 0;
  bottom: 0;
  width: 100%;
  max-width: none;
}

/* Inline mode */
.mtchat--inline {
  position: relative;
}
</style>
