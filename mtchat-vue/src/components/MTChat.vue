<script setup lang="ts">
/**
 * MTChat Vue Component
 *
 * Two display modes:
 * - Full mode: Dialog list with tabs (My/Available) + chat area
 * - Inline mode: Single chat for a business object
 */

import { computed, ref, watch, nextTick, onMounted, onUnmounted } from 'vue'
import { useChat } from '../composables/useChat'
import { useFileUpload } from '../composables/useFileUpload'
import { provideI18n } from '../i18n'
import type { MTChatConfig, ChatMode, DialogListItem, Message, Attachment } from '../types'
import AttachmentPreview from './chat/AttachmentPreview.vue'
import AttachmentList from './chat/AttachmentList.vue'
import FileViewer from './chat/FileViewer.vue'
import ChatInfoPanel from './chat/ChatInfoPanel.vue'
import JoinDialog from './chat/JoinDialog.vue'

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

// i18n setup - provide locale to child components and get i18n for this component
const { t, tt, formatDateDivider, localeRef } = provideI18n(props.config.locale ?? 'ru')

// Watch for locale changes in config
watch(
  () => props.config.locale,
  (newLocale) => {
    if (newLocale && newLocale !== localeRef.value) {
      localeRef.value = newLocale
    }
  }
)

// Chat composable
const chat = useChat({
  config: props.config,
  dialogId: props.dialogId,
  objectId: props.objectId,
  objectType: props.objectType,
})

// Dialog ID ref for file uploads
const currentDialogId = computed(() => chat.currentDialog.value?.id)

// File upload composable
const fileUpload = useFileUpload({
  dialogId: currentDialogId,
  api: chat.api,
})

// Local state
const messageInput = ref('')
const messagesContainer = ref<HTMLElement | null>(null)
const fileInputRef = ref<HTMLInputElement | null>(null)
const activeTab = ref<'participating' | 'available'>('participating')
const searchInput = ref('')

// Debounce timer for search
let searchTimeout: ReturnType<typeof setTimeout> | null = null

// Read tracking state
let readTimeout: ReturnType<typeof setTimeout> | null = null

// File viewer state
const showFileViewer = ref(false)
const fileViewerIndex = ref(0)

// Scroll button state
const showScrollButton = ref(false)

// Sticky date state
const stickyDate = ref<string | null>(null)
const hiddenDividerDate = ref<string | null>(null)

// Info panel and menu state
const showInfoPanel = ref(false)
const showHeaderMenu = ref(false)

// Join dialog state
const showJoinDialog = ref(false)
const isJoining = ref(false)

// Collect all attachments from all messages
const allAttachments = computed(() => {
  const attachments: Attachment[] = []
  for (const message of chat.messages.value) {
    if (message.attachments) {
      attachments.push(...message.attachments)
    }
  }
  return attachments
})

// Computed
const isInlineMode = computed(() => props.mode === 'inline')
const hasDialog = computed(() => chat.currentDialog.value !== null)
const canSendMessage = computed(() =>
  hasDialog.value && chat.currentDialog.value?.i_am_participant
)
const canJoin = computed(() =>
  chat.currentDialog.value?.can_join &&
  !chat.currentDialog.value?.i_am_participant
)

// Check if we have content to send (text or attachments)
const hasContentToSend = computed(() =>
  messageInput.value.trim() || fileUpload.hasUploaded.value
)

const dialogTitle = computed(() => {
  const dialog = chat.currentDialog.value
  if (!dialog) return ''
  return dialog.title || `${dialog.object_type}/${dialog.object_id}`
})

const currentDialogsList = computed(() =>
  activeTab.value === 'participating'
    ? chat.participatingDialogs.value
    : chat.availableDialogs.value
)

// Watch for connection changes
watch(
  () => chat.isConnected.value,
  (connected) => {
    if (connected) {
      emit('connected')
      // Load dialogs when connected (full mode)
      if (!isInlineMode.value) {
        chat.loadParticipatingDialogs()
        chat.loadAvailableDialogs()
      }
    } else {
      emit('disconnected')
    }
  }
)

// Watch for errors
watch(
  () => chat.error.value,
  (error) => {
    if (error) {
      emit('error', error)
    }
  }
)

// Auto-scroll on new messages
watch(
  () => chat.messages.value.length,
  async () => {
    await nextTick()
    scrollToBottom()
  }
)

// Debounced search - reload dialogs when search input changes
watch(searchInput, (newValue) => {
  if (searchTimeout) {
    clearTimeout(searchTimeout)
  }
  searchTimeout = setTimeout(() => {
    chat.setSearchQuery(newValue)
    chat.loadParticipatingDialogs()
    chat.loadAvailableDialogs()
  }, 300)
})

// Clear search input and reload dialogs
function clearSearch() {
  searchInput.value = ''
  chat.setSearchQuery('')
  chat.loadParticipatingDialogs()
  chat.loadAvailableDialogs()
}

// Check if scrolled to bottom and mark as read
function handleScroll() {
  if (!messagesContainer.value) return

  const container = messagesContainer.value
  const distanceFromBottom = container.scrollHeight - container.scrollTop - container.clientHeight
  const isAtBottom = distanceFromBottom < 50

  // Show/hide scroll to bottom button
  showScrollButton.value = distanceFromBottom > 200

  // Update sticky date - find date divider that's scrolled past top
  const dateDividers = container.querySelectorAll('.mtchat__date-divider')
  const containerRect = container.getBoundingClientRect()
  let activeDateText: string | null = null
  let hiddenDate: string | null = null
  let hideSticky = false

  const dividerArray = Array.from(dateDividers)

  for (let i = 0; i < dividerArray.length; i++) {
    const divider = dividerArray[i]
    const rect = divider.getBoundingClientRect()
    const relativeTop = rect.top - containerRect.top

    // If divider is scrolled above container top
    if (relativeTop < 0) {
      activeDateText = divider.textContent?.trim() || null
      hiddenDate = activeDateText // Hide in-flow divider that's past top

      // Check if NEXT divider is close to top
      const nextDivider = dividerArray[i + 1]
      if (nextDivider) {
        const nextRect = nextDivider.getBoundingClientRect()
        const nextRelativeTop = nextRect.top - containerRect.top
        // If next divider is near top (visible or about to be), hide sticky
        if (nextRelativeTop >= 0 && nextRelativeTop < 60) {
          hideSticky = true
        }
      }
    }
  }

  // Hide sticky date only if next divider is close to top
  stickyDate.value = hideSticky ? null : activeDateText
  hiddenDividerDate.value = hideSticky ? null : hiddenDate

  // Mark as read logic
  if (chat.firstUnreadMessageId.value) {
    if (isAtBottom) {
      // Start timer to mark as read after 1 second
      if (!readTimeout) {
        readTimeout = setTimeout(() => {
          chat.markAsRead()
          readTimeout = null
        }, 1000)
      }
    } else {
      // Cancel if scrolled away
      if (readTimeout) {
        clearTimeout(readTimeout)
        readTimeout = null
      }
    }
  }
}

// Methods
function scrollToBottom() {
  if (messagesContainer.value) {
    messagesContainer.value.scrollTop = messagesContainer.value.scrollHeight
  }
}

async function handleSendMessage() {
  if (!hasContentToSend.value || !canSendMessage.value) return

  const content = messageInput.value.trim()
  const attachments = fileUpload.getUploadedAttachments()

  messageInput.value = ''
  fileUpload.clearAll()

  try {
    const message = await chat.sendMessage(
      content,
      attachments.length > 0 ? attachments : undefined
    )
    if (message) {
      emit('message-sent', message)
    }
  } catch (e) {
    // Error already handled in composable
  }
}

async function handleSelectDialog(dialog: DialogListItem) {
  await chat.selectDialog(dialog.id)
  emit('dialog-selected', dialog)
}

function handleJoinDialog() {
  if (!chat.currentDialog.value) return
  // Show the join dialog modal
  showJoinDialog.value = true
}

async function confirmJoinDialog(profile: { display_name: string; company: string; email?: string; phone?: string }) {
  if (!chat.currentDialog.value) return

  const dialogId = chat.currentDialog.value.id

  try {
    isJoining.value = true
    await chat.joinDialog(dialogId, profile)
    showJoinDialog.value = false

    // Switch to "My Chats" tab (full mode only)
    if (!isInlineMode.value) {
      activeTab.value = 'participating'
    }

    // Reload dialog to get fresh state (participants, etc.)
    await chat.selectDialog(dialogId)

    emit('dialog-joined', dialogId)
  } catch (e) {
    // Error already handled
  } finally {
    isJoining.value = false
  }
}

async function handleLeaveDialog() {
  if (!chat.currentDialog.value) return

  const dialogId = chat.currentDialog.value.id
  try {
    await chat.leaveDialog(dialogId)

    // Reload available dialogs to get the chat back (if user still has access)
    // Stay on "My Chats" tab - user will see "Select a chat" message
    if (!isInlineMode.value) {
      await chat.loadAvailableDialogs()
    }

    emit('dialog-left', dialogId)
  } catch (e) {
    // Error already handled
  }
}

function formatTime(dateString: string): string {
  const date = new Date(dateString)
  return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
}

function getDateKey(dateString: string): string {
  const date = new Date(dateString)
  return `${date.getFullYear()}-${date.getMonth()}-${date.getDate()}`
}

// formatDateDivider is now provided by useI18n

function shouldShowDateDivider(message: Message, index: number): boolean {
  if (index === 0) return true
  const prevMessage = chat.messages.value[index - 1]
  return getDateKey(message.sent_at) !== getDateKey(prevMessage.sent_at)
}

// File handling
function handleFileSelect() {
  fileInputRef.value?.click()
}

function handleFileChange(event: Event) {
  const input = event.target as HTMLInputElement
  if (input.files && input.files.length > 0) {
    fileUpload.addFiles(Array.from(input.files))
    input.value = '' // Reset input for same file selection
  }
}

// File viewer handling
function openFileViewer(attachment: Attachment) {
  // Find index of this attachment in allAttachments
  const index = allAttachments.value.findIndex((a) => a.id === attachment.id)
  if (index !== -1) {
    fileViewerIndex.value = index
    showFileViewer.value = true
  }
}

function openGalleryAtIndex(message: Message, imageIndex: number) {
  // Find all image attachments in this message
  const imageAttachments = message.attachments?.filter((a) =>
    a.content_type.startsWith('image/')
  ) || []

  if (imageAttachments.length > 0) {
    // Get the specific image clicked
    const clickedImage = imageAttachments[imageIndex]
    if (clickedImage) {
      openFileViewer(clickedImage)
    }
  }
}

function closeFileViewer() {
  showFileViewer.value = false
}

// Reply helpers
function truncateText(text: string, maxLength: number): string {
  if (text.length <= maxLength) return text
  return text.slice(0, maxLength) + '...'
}

function getQuotedText(messageId: string): string {
  const msg = chat.messages.value.find((m) => m.id === messageId)
  if (!msg) return t.value.chat.messageDeleted
  return truncateText(msg.content, 60)
}

/**
 * Get the display name for a user in the current dialog
 */
function getSenderDisplayName(senderId: string): string {
  // Check if it's the current user
  const isCurrentUser = senderId === props.config.userId

  // Find participant by user_id
  const participant = chat.participants.value.find((p) => p.user_id === senderId)

  if (participant?.display_name) {
    // Use display_name with "(You)" suffix for current user
    return isCurrentUser ? `${participant.display_name} ${t.value.user.youBadge}` : participant.display_name
  }

  // Fallback if participant not found or no display_name
  return isCurrentUser ? t.value.user.you : senderId.slice(0, 8)
}

function getMessageAuthor(messageId: string): string {
  const msg = chat.messages.value.find((m) => m.id === messageId)
  if (!msg) return '...'
  return getSenderDisplayName(msg.sender_id)
}

function scrollToMessage(messageId: string) {
  const messageEl = messagesContainer.value?.querySelector(
    `[data-message-id="${messageId}"]`
  )
  if (messageEl) {
    messageEl.scrollIntoView({ behavior: 'smooth', block: 'center' })
    // Highlight effect
    messageEl.classList.add('mtchat__message--highlight')
    setTimeout(() => {
      messageEl.classList.remove('mtchat__message--highlight')
    }, 2000)
  }
}

function handleScrollToBottom() {
  if (messagesContainer.value) {
    messagesContainer.value.scrollTo({
      top: messagesContainer.value.scrollHeight,
      behavior: 'smooth'
    })
  }
}

// Keyboard handler for Esc
function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape' && chat.replyToMessage.value) {
    chat.clearReplyTo()
  }
}

// Lifecycle - keyboard handlers
onMounted(() => {
  document.addEventListener('keydown', handleKeydown)
})

// Cleanup
onUnmounted(() => {
  if (readTimeout) {
    clearTimeout(readTimeout)
    readTimeout = null
  }
  if (searchTimeout) {
    clearTimeout(searchTimeout)
    searchTimeout = null
  }
  document.removeEventListener('keydown', handleKeydown)
})

// Expose for parent access
defineExpose({
  client: chat,
  messages: chat.messages,
  participatingDialogs: chat.participatingDialogs,
  availableDialogs: chat.availableDialogs,
  currentDialog: chat.currentDialog,
  isConnected: chat.isConnected,
  selectDialog: chat.selectDialog,
  sendMessage: chat.sendMessage,
  joinDialog: chat.joinDialog,
  leaveDialog: chat.leaveDialog,
})
</script>

<template>
  <div :class="['mtchat', `mtchat--${theme}`, { 'mtchat--inline': isInlineMode }]">
    <!-- Sidebar (Full mode only) -->
    <aside v-if="!isInlineMode && showSidebar" class="mtchat__sidebar">
      <!-- Search -->
      <div class="mtchat__search">
        <input
          v-model="searchInput"
          type="text"
          :placeholder="t.search.placeholder"
          class="mtchat__search-input"
          @keydown.esc="clearSearch"
        />
        <button
          v-if="searchInput"
          class="mtchat__search-clear"
          type="button"
          @click="clearSearch"
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M18 6L6 18M6 6l12 12"/>
          </svg>
        </button>
      </div>

      <!-- Tabs -->
      <div class="mtchat__tabs">
        <button
          :class="['mtchat__tab', { 'mtchat__tab--active': activeTab === 'participating' }]"
          @click="activeTab = 'participating'"
        >
          {{ t.tabs.myChats }}
          <span v-if="chat.participatingDialogs.value.length" class="mtchat__tab-count">
            {{ chat.participatingDialogs.value.length }}
          </span>
        </button>
        <button
          :class="['mtchat__tab', { 'mtchat__tab--active': activeTab === 'available' }]"
          @click="activeTab = 'available'"
        >
          {{ t.tabs.available }}
          <span v-if="chat.availableDialogs.value.length" class="mtchat__tab-count">
            {{ chat.availableDialogs.value.length }}
          </span>
        </button>
      </div>

      <!-- Dialog List -->
      <div class="mtchat__dialog-list">
        <div
          v-for="dialog in currentDialogsList"
          :key="dialog.id"
          :class="['mtchat__dialog-item', { 'mtchat__dialog-item--active': chat.currentDialog.value?.id === dialog.id }]"
          @click="handleSelectDialog(dialog)"
        >
          <div class="mtchat__dialog-title">
            {{ dialog.title || `${dialog.object_type}/${dialog.object_id}` }}
          </div>
          <div class="mtchat__dialog-meta">
            <span class="mtchat__dialog-type">{{ dialog.object_type }}</span>
            <span class="mtchat__dialog-participants">
              {{ tt('chat.participants', { count: dialog.participants_count }) }}
            </span>
          </div>
          <span v-if="dialog.unread_count && dialog.unread_count > 0" class="mtchat__unread-badge">
            {{ dialog.unread_count > 99 ? '99+' : dialog.unread_count }}
          </span>
        </div>

        <div v-if="currentDialogsList.length === 0" class="mtchat__empty">
          {{ searchInput
            ? t.search.noResults
            : (activeTab === 'participating' ? t.chat.noActiveChats : t.chat.noAvailableChats)
          }}
        </div>
      </div>
    </aside>

    <!-- Main Chat Area -->
    <main class="mtchat__main">
      <!-- Header -->
      <header v-if="showHeader && hasDialog" class="mtchat__header">
        <button
          class="mtchat__header-info"
          @click="showInfoPanel = true"
          :title="t.tooltips.chatInfo"
        >
          <h2 class="mtchat__header-title">{{ dialogTitle }}</h2>
          <div class="mtchat__header-meta">
            <span class="mtchat__header-participants">
              {{ tt('chat.participants', { count: chat.currentDialog.value?.participants_count || 0 }) }}
            </span>
            <span :class="['mtchat__status', { 'mtchat__status--connected': chat.isConnected.value }]">
              {{ chat.isConnected.value ? t.status.connected : t.status.disconnected }}
            </span>
          </div>
        </button>
        <div class="mtchat__header-actions">
          <!-- Join button for non-participants -->
          <button
            v-if="canJoin"
            class="mtchat__btn mtchat__btn--primary"
            @click="handleJoinDialog"
            :disabled="chat.isLoading.value"
          >
            {{ t.buttons.join }}
          </button>
          <!-- Menu button for participants -->
          <div v-else-if="chat.currentDialog.value?.i_am_participant" class="mtchat__menu-container">
            <button
              class="mtchat__menu-button"
              @click="showHeaderMenu = !showHeaderMenu"
              :title="t.tooltips.menu"
            >
              <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
                <circle cx="12" cy="5" r="2"/>
                <circle cx="12" cy="12" r="2"/>
                <circle cx="12" cy="19" r="2"/>
              </svg>
            </button>
            <!-- Dropdown menu -->
            <div v-if="showHeaderMenu" class="mtchat__menu-dropdown" @click.stop>
              <button
                class="mtchat__menu-item"
                @click="showInfoPanel = true; showHeaderMenu = false"
              >
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
                  <circle cx="12" cy="12" r="10"/>
                  <path d="M12 16v-4"/>
                  <path d="M12 8h.01"/>
                </svg>
                {{ t.buttons.info }}
              </button>
              <button
                class="mtchat__menu-item mtchat__menu-item--danger"
                @click="handleLeaveDialog(); showHeaderMenu = false"
                :disabled="chat.isLoading.value"
              >
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M9 21H5a2 2 0 01-2-2V5a2 2 0 012-2h4"/>
                  <polyline points="16 17 21 12 16 7"/>
                  <line x1="21" y1="12" x2="9" y2="12"/>
                </svg>
                {{ t.buttons.leaveChat }}
              </button>
            </div>
          </div>
        </div>
      </header>
      <!-- Click outside to close menu -->
      <div
        v-if="showHeaderMenu"
        class="mtchat__menu-backdrop"
        @click="showHeaderMenu = false"
      ></div>

      <!-- No Dialog Selected -->
      <div v-if="!hasDialog" class="mtchat__no-dialog">
        <p v-if="isInlineMode">{{ t.chat.noChatForObject }}</p>
        <p v-else>{{ t.chat.selectChat }}</p>
      </div>

      <!-- Join Required (not a participant yet) -->
      <div v-else-if="!chat.currentDialog.value?.i_am_participant" class="mtchat__join-required">
        <div class="mtchat__join-required-content">
          <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <rect x="3" y="11" width="18" height="11" rx="2" ry="2"/>
            <path d="M7 11V7a5 5 0 0110 0v4"/>
          </svg>
          <p>{{ t.chat.joinRequired }}</p>
          <button
            v-if="canJoin"
            class="mtchat__btn mtchat__btn--primary"
            @click="handleJoinDialog"
            :disabled="chat.isLoading.value"
          >
            {{ t.buttons.join }}
          </button>
        </div>
      </div>

      <!-- Messages (only for participants) -->
      <div v-else ref="messagesContainer" class="mtchat__messages" @scroll="handleScroll">
        <!-- Floating sticky date (appears when scrolled 40px+) -->
        <div v-if="stickyDate" class="mtchat__sticky-date">
          <span>{{ stickyDate }}</span>
        </div>

        <template v-for="(message, index) in chat.messages.value" :key="message.id">
          <!-- Date divider (in-flow, faded when sticky is showing same date) -->
          <div
            v-if="shouldShowDateDivider(message, index)"
            :class="['mtchat__date-divider', { 'mtchat__date-divider--hidden': formatDateDivider(message.sent_at) === hiddenDividerDate }]"
          >
            <span>{{ formatDateDivider(message.sent_at) }}</span>
          </div>

          <!-- Unread divider -->
          <div
            v-if="message.id === chat.firstUnreadMessageId.value"
            class="mtchat__unread-divider"
          >
            <span>{{ t.chat.newMessages }}</span>
          </div>

          <div
            :data-message-id="message.id"
            class="mtchat__message"
          >
            <!-- Message actions (top-right, visible on hover) -->
            <div class="mtchat__message-actions">
              <button
                class="mtchat__action-btn"
                :title="t.tooltips.reply"
                @click.stop="chat.setReplyTo(message)"
              >
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M9 14L4 9l5-5"/>
                  <path d="M4 9h10.5a5.5 5.5 0 0 1 5.5 5.5v0a5.5 5.5 0 0 1-5.5 5.5H11"/>
                </svg>
              </button>
            </div>

            <!-- Quoted message (if reply) -->
            <div
              v-if="message.reply_to_id"
              class="mtchat__quoted-message"
              @click.stop="scrollToMessage(message.reply_to_id)"
            >
              <div class="mtchat__quoted-indicator"></div>
              <div class="mtchat__quoted-content">
                <div class="mtchat__quoted-author">
                  {{ getMessageAuthor(message.reply_to_id) }}
                </div>
                <div class="mtchat__quoted-text">
                  {{ getQuotedText(message.reply_to_id) }}
                </div>
              </div>
            </div>

            <!-- Header: sender name + time -->
            <div class="mtchat__message-header">
              <span class="mtchat__message-sender">
                {{ getSenderDisplayName(message.sender_id) }}
              </span>
              <span class="mtchat__message-time">{{ formatTime(message.sent_at) }}</span>
            </div>

            <!-- Content -->
            <div v-if="message.content" class="mtchat__message-content">{{ message.content }}</div>

            <!-- Attachments -->
            <AttachmentList
              v-if="message.attachments && message.attachments.length > 0"
              :attachments="message.attachments"
              @open-gallery="(index) => openGalleryAtIndex(message, index)"
              @open-file="openFileViewer"
            />
          </div>
        </template>

        <div v-if="chat.messages.value.length === 0" class="mtchat__empty">
          {{ t.chat.noMessages }}
        </div>
      </div>

      <!-- Scroll to bottom button (fixed position, outside scroll container) -->
      <button
        v-if="showScrollButton && hasDialog"
        class="mtchat__scroll-bottom"
        :title="t.tooltips.scrollDown"
        @click="handleScrollToBottom"
      >
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <polyline points="6 9 12 15 18 9"/>
        </svg>
      </button>

      <!-- Input Area -->
      <div v-if="hasDialog" class="mtchat__input-area">
        <div v-if="!canSendMessage && canJoin" class="mtchat__join-prompt">
          <p>{{ t.chat.joinToSend }}</p>
          <button class="mtchat__btn mtchat__btn--primary" @click="handleJoinDialog">
            {{ t.buttons.join }}
          </button>
        </div>
        <template v-else-if="canSendMessage">
          <!-- Reply Preview -->
          <div v-if="chat.replyToMessage.value" class="mtchat__reply-preview">
            <div class="mtchat__reply-indicator"></div>
            <div class="mtchat__reply-content">
              <div class="mtchat__reply-author">
                {{ getSenderDisplayName(chat.replyToMessage.value.sender_id) }}
              </div>
              <div class="mtchat__reply-text">
                {{ truncateText(chat.replyToMessage.value.content, 100) }}
              </div>
            </div>
            <button class="mtchat__reply-cancel" @click="chat.clearReplyTo()">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M18 6L6 18M6 6l12 12"/>
              </svg>
            </button>
          </div>

          <!-- Attachment Preview -->
          <AttachmentPreview
            :attachments="fileUpload.pendingAttachments.value"
            @remove="fileUpload.removeAttachment"
            @retry="fileUpload.retryUpload"
          />

          <!-- Input Form -->
          <form class="mtchat__input-form" @submit.prevent="handleSendMessage">
            <!-- Hidden file input -->
            <input
              ref="fileInputRef"
              type="file"
              multiple
              class="mtchat__file-input"
              @change="handleFileChange"
            />

            <!-- Attach button -->
            <button
              type="button"
              class="mtchat__btn mtchat__btn--attach"
              :title="t.input.attachFiles"
              :disabled="chat.isLoading.value || fileUpload.isUploading.value"
              @click="handleFileSelect"
            >
              <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M21.44 11.05l-9.19 9.19a6 6 0 0 1-8.49-8.49l9.19-9.19a4 4 0 0 1 5.66 5.66l-9.2 9.19a2 2 0 0 1-2.83-2.83l8.49-8.48" />
              </svg>
            </button>

            <!-- Message input -->
            <input
              v-model="messageInput"
              type="text"
              class="mtchat__input"
              :placeholder="t.input.placeholder"
              :disabled="chat.isLoading.value"
            />

            <!-- Send button -->
            <button
              type="submit"
              class="mtchat__btn mtchat__btn--send"
              :disabled="!hasContentToSend || chat.isLoading.value || fileUpload.isUploading.value"
            >
              {{ t.buttons.send }}
            </button>
          </form>
        </template>
      </div>
    </main>

    <!-- Info Panel (side column in full mode, overlay in inline mode) -->
    <Transition name="mtchat-info-panel">
      <aside
        v-if="showInfoPanel && hasDialog"
        :class="['mtchat__info-panel', { 'mtchat__info-panel--inline': isInlineMode }]"
      >
        <ChatInfoPanel
          :dialog-title="dialogTitle"
          :object-type="chat.currentDialog.value?.object_type"
          :object-id="chat.currentDialog.value?.object_id"
          :participants="chat.participants.value"
          :current-user-id="config.userId"
          @close="showInfoPanel = false"
        />
      </aside>
    </Transition>

    <!-- Unified File Viewer Modal -->
    <FileViewer
      :show="showFileViewer"
      :files="allAttachments"
      :initial-index="fileViewerIndex"
      @close="closeFileViewer"
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
      @join="confirmJoinDialog"
    />
  </div>
</template>

<style scoped>
.mtchat {
  display: flex;
  height: 100%;
  min-height: 400px;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  font-size: 14px;
  border: 1px solid var(--mtchat-border, #e0e0e0);
  border-radius: 8px;
  overflow: hidden;
  position: relative;
}

.mtchat--inline {
  border-radius: 4px;
  min-height: 0;
  border: none;
}

/* Light theme (PrimeVue Lara Light Blue) */
.mtchat--light {
  --mtchat-bg: #ffffff;
  --mtchat-bg-secondary: #f8fafc;
  --mtchat-bg-hover: #f1f5f9;
  --mtchat-text: #334155;
  --mtchat-text-secondary: #64748b;
  --mtchat-border: #e2e8f0;
  --mtchat-primary: #3B82F6;
  --mtchat-primary-hover: #2563eb;
  --mtchat-primary-text: #ffffff;
}

/* Dark theme (PrimeVue Lara Dark Blue) */
.mtchat--dark {
  --mtchat-bg: #1f2937;
  --mtchat-bg-secondary: #111827;
  --mtchat-bg-hover: #374151;
  --mtchat-text: #f8fafc;
  --mtchat-text-secondary: #94a3b8;
  --mtchat-border: #374151;
  --mtchat-primary: #60a5fa;
  --mtchat-primary-hover: #3b82f6;
  --mtchat-primary-text: #1f2937;
}

/* Sidebar */
.mtchat__sidebar {
  width: 280px;
  border-right: 1px solid var(--mtchat-border);
  background: var(--mtchat-bg);
  display: flex;
  flex-direction: column;
}

/* Search */
.mtchat__search {
  padding: 12px;
  border-bottom: 1px solid var(--mtchat-border);
  display: flex;
  gap: 8px;
  position: relative;
}

.mtchat__search-input {
  flex: 1;
  padding: 8px 12px;
  padding-right: 32px;
  border: 1px solid var(--mtchat-border);
  border-radius: 6px;
  background: var(--mtchat-bg);
  color: var(--mtchat-text);
  font-size: 14px;
  outline: none;
}

.mtchat__search-input:focus {
  border-color: var(--mtchat-primary);
}

.mtchat__search-input::placeholder {
  color: var(--mtchat-text-secondary);
}

.mtchat__search-clear {
  position: absolute;
  right: 20px;
  top: 50%;
  transform: translateY(-50%);
  background: none;
  border: none;
  color: var(--mtchat-text-secondary);
  cursor: pointer;
  padding: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 4px;
}

.mtchat__search-clear:hover {
  color: var(--mtchat-text);
  background: var(--mtchat-bg-hover);
}

/* Tabs */
.mtchat__tabs {
  display: flex;
  border-bottom: 1px solid var(--mtchat-border);
}

.mtchat__tab {
  flex: 1;
  padding: 12px 8px;
  background: none;
  border: none;
  border-bottom: 2px solid transparent;
  color: var(--mtchat-text-secondary);
  cursor: pointer;
  font-size: 13px;
  font-weight: 500;
  transition: all 0.2s;
}

.mtchat__tab:hover {
  color: var(--mtchat-text);
  background: var(--mtchat-bg-secondary);
}

.mtchat__tab--active {
  color: var(--mtchat-primary);
  border-bottom-color: var(--mtchat-primary);
}

.mtchat__tab-count {
  margin-left: 4px;
  padding: 2px 6px;
  background: var(--mtchat-bg-secondary);
  border-radius: 10px;
  font-size: 11px;
}

/* Dialog List */
.mtchat__dialog-list {
  flex: 1;
  overflow-y: auto;
}

.mtchat__dialog-item {
  padding: 12px 16px;
  border-bottom: 1px solid var(--mtchat-border);
  cursor: pointer;
  transition: background 0.2s;
  position: relative;
}

.mtchat__dialog-item:hover {
  background: var(--mtchat-bg-secondary);
}

.mtchat__dialog-item--active {
  background: var(--mtchat-bg-secondary);
  border-left: 3px solid var(--mtchat-primary);
}

.mtchat__dialog-title {
  font-weight: 500;
  color: var(--mtchat-text);
  margin-bottom: 4px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.mtchat__dialog-meta {
  font-size: 12px;
  color: var(--mtchat-text-secondary);
  display: flex;
  gap: 8px;
}

.mtchat__dialog-type {
  background: var(--mtchat-bg-secondary);
  padding: 2px 6px;
  border-radius: 4px;
}

.mtchat__dialog-badge {
  position: absolute;
  top: 8px;
  right: 8px;
  font-size: 10px;
  padding: 2px 6px;
  background: var(--mtchat-primary);
  color: white;
  border-radius: 4px;
}

.mtchat__unread-badge {
  position: absolute;
  top: 50%;
  right: 12px;
  transform: translateY(-50%);
  background: #007AFF;
  color: white;
  border-radius: 10px;
  padding: 2px 8px;
  font-size: 12px;
  font-weight: 600;
  min-width: 20px;
  text-align: center;
}

/* Main Area */
.mtchat__main {
  flex: 1;
  display: flex;
  flex-direction: column;
  background: var(--mtchat-bg);
  min-width: 0;
  min-height: 0;
  position: relative;
}

/* Header */
.mtchat__header {
  padding: 12px 16px;
  border-bottom: 1px solid var(--mtchat-border);
  display: flex;
  align-items: center;
  justify-content: space-between;
  background: var(--mtchat-bg);
}

.mtchat__header-title {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  color: var(--mtchat-text);
}

.mtchat__header-meta {
  display: flex;
  gap: 12px;
  font-size: 12px;
  color: var(--mtchat-text-secondary);
  margin-top: 4px;
}

.mtchat__status {
  display: flex;
  align-items: center;
  gap: 4px;
}

.mtchat__status::before {
  content: '';
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: #f44336;
}

.mtchat__status--connected::before {
  background: #4caf50;
}

/* Header info as button */
button.mtchat__header-info {
  background: none;
  border: none;
  padding: 0;
  margin: 0;
  cursor: pointer;
  text-align: left;
  border-radius: 8px;
  transition: background-color 0.2s;
  outline: none;
}

button.mtchat__header-info:hover,
button.mtchat__header-info:focus {
  background: var(--mtchat-hover);
  padding: 4px 8px;
  margin: -4px -8px;
}

/* Menu container */
.mtchat__menu-container {
  position: relative;
}

.mtchat__menu-button {
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  background: none;
  border-radius: 8px;
  cursor: pointer;
  color: var(--mtchat-text-secondary);
  transition: background-color 0.2s, color 0.2s;
}

.mtchat__menu-button:hover {
  background: var(--mtchat-hover);
  color: var(--mtchat-text);
}

.mtchat__menu-dropdown {
  position: absolute;
  top: 100%;
  right: 0;
  margin-top: 4px;
  min-width: 180px;
  background: var(--mtchat-bg);
  border: 1px solid var(--mtchat-border);
  border-radius: 8px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.15);
  z-index: 100;
  overflow: hidden;
}

.mtchat__menu-item {
  width: 100%;
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 14px;
  border: none;
  background: none;
  cursor: pointer;
  font-size: 14px;
  color: var(--mtchat-text);
  text-align: left;
  transition: background-color 0.15s;
}

.mtchat__menu-item:hover {
  background: var(--mtchat-hover);
}

.mtchat__menu-item svg {
  flex-shrink: 0;
}

.mtchat__menu-item--danger {
  color: #f44336;
}

.mtchat__menu-item--danger:hover {
  background: rgba(244, 67, 54, 0.1);
}

.mtchat__menu-backdrop {
  position: fixed;
  inset: 0;
  z-index: 99;
}

/* Messages */
.mtchat__messages {
  flex: 1;
  overflow-x: hidden;
  overflow-y: auto;
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 4px;
  position: relative;
}

/* Floating sticky date (fixed at top when scrolled) */
.mtchat__sticky-date {
  position: sticky;
  top: 0;
  z-index: 20;
  display: flex;
  justify-content: center;
  padding: 8px 0;
  pointer-events: none;
}

.mtchat__sticky-date span {
  padding: 4px 12px;
  background: var(--mtchat-bg);
  border: 1px solid var(--mtchat-border);
  border-radius: 12px;
  font-size: 12px;
  font-weight: 500;
  color: var(--mtchat-text-secondary);
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
  pointer-events: auto;
}

/* Date divider (in-flow) */
.mtchat__date-divider {
  display: flex;
  justify-content: center;
  padding: 8px 0;
  margin: 8px 0;
}

.mtchat__date-divider--hidden {
  visibility: hidden;
}

.mtchat__date-divider span {
  padding: 4px 12px;
  background: var(--mtchat-bg-secondary);
  border-radius: 12px;
  font-size: 12px;
  font-weight: 500;
  color: var(--mtchat-text-secondary);
}

/* Message (list-style, full width) */
.mtchat__message {
  position: relative;
  padding: 8px 12px;
  border-radius: 4px;
  transition: background 0.15s;
}

.mtchat__message:hover {
  background: var(--mtchat-bg-hover);
}

/* Message header: sender + datetime */
.mtchat__message-header {
  display: flex;
  align-items: baseline;
  gap: 8px;
  margin-bottom: 4px;
}

.mtchat__message-sender {
  font-weight: 600;
  font-size: 14px;
  color: var(--mtchat-text);
}

.mtchat__message-time {
  font-size: 12px;
  color: var(--mtchat-text-secondary);
}

/* Message content */
.mtchat__message-content {
  color: var(--mtchat-text);
  font-size: 14px;
  line-height: 1.5;
  white-space: pre-wrap;
  word-wrap: break-word;
}

/* Message actions (top-right, visible on hover) */
.mtchat__message-actions {
  position: absolute;
  top: 8px;
  right: 8px;
  display: flex;
  gap: 4px;
  opacity: 0;
  transition: opacity 0.15s;
}

.mtchat__message:hover .mtchat__message-actions {
  opacity: 1;
}

.mtchat__action-btn {
  width: 28px;
  height: 28px;
  border: none;
  border-radius: 4px;
  background: var(--mtchat-bg);
  color: var(--mtchat-text-secondary);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background 0.15s, color 0.15s;
}

.mtchat__action-btn:hover {
  background: var(--mtchat-bg-secondary);
  color: var(--mtchat-primary);
}

/* Quoted message */
.mtchat__quoted-message {
  display: flex;
  gap: 8px;
  padding: 8px 12px;
  margin-bottom: 8px;
  background: var(--mtchat-bg-secondary);
  border-left: 3px solid var(--mtchat-primary);
  border-radius: 0 4px 4px 0;
  cursor: pointer;
  transition: background 0.15s;
}

.mtchat__quoted-message:hover {
  background: var(--mtchat-border);
}

.mtchat__quoted-indicator {
  display: none;
}

.mtchat__quoted-content {
  flex: 1;
  min-width: 0;
}

.mtchat__quoted-author {
  font-size: 12px;
  font-weight: 600;
  color: var(--mtchat-primary);
  margin-bottom: 2px;
}

.mtchat__quoted-text {
  font-size: 13px;
  color: var(--mtchat-text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

/* Message highlight animation */
.mtchat__message--highlight {
  animation: highlight-pulse 2s ease-out;
}

@keyframes highlight-pulse {
  0% { background: rgba(0, 122, 255, 0.3); }
  100% { background: transparent; }
}

/* Scroll to bottom button */
.mtchat__scroll-bottom {
  position: absolute;
  bottom: 84px; /* input area (~68px) + 16px spacing */
  right: 16px;
  width: 36px;
  height: 36px;
  border: 1px solid var(--mtchat-border);
  border-radius: 50%;
  background: var(--mtchat-bg);
  color: var(--mtchat-text-secondary);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background 0.15s, color 0.15s, border-color 0.15s;
  z-index: 100;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.mtchat__scroll-bottom:hover {
  background: var(--mtchat-bg-secondary);
  color: var(--mtchat-primary);
  border-color: var(--mtchat-primary);
}

.mtchat__message-sender {
  font-size: 11px;
  color: var(--mtchat-text-secondary);
  margin-bottom: 2px;
}

.mtchat__message-content {
  color: var(--mtchat-text);
  word-wrap: break-word;
}

.mtchat__message-time {
  font-size: 10px;
  color: var(--mtchat-text-secondary);
  text-align: right;
  margin-top: 4px;
}

/* Unread Divider */
.mtchat__unread-divider {
  display: flex;
  align-items: center;
  margin: 16px 0;
  gap: 12px;
}

.mtchat__unread-divider::before,
.mtchat__unread-divider::after {
  content: '';
  flex: 1;
  height: 1px;
  background: rgba(0, 122, 255, 0.3);
}

.mtchat__unread-divider span {
  color: #007AFF;
  font-size: 13px;
  font-weight: 500;
  white-space: nowrap;
}

/* Reply Preview */
.mtchat__reply-preview {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  background: var(--mtchat-bg-secondary);
  border-radius: 8px;
  margin-bottom: 8px;
}

.mtchat__reply-indicator {
  width: 3px;
  height: 100%;
  min-height: 32px;
  background: var(--mtchat-primary);
  border-radius: 2px;
}

.mtchat__reply-content {
  flex: 1;
  min-width: 0;
}

.mtchat__reply-author {
  font-size: 12px;
  font-weight: 600;
  color: var(--mtchat-primary);
}

.mtchat__reply-text {
  font-size: 13px;
  color: var(--mtchat-text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.mtchat__reply-cancel {
  width: 24px;
  height: 24px;
  border: none;
  background: transparent;
  color: var(--mtchat-text-secondary);
  cursor: pointer;
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.mtchat__reply-cancel:hover {
  background: var(--mtchat-bg);
  color: var(--mtchat-text);
}

/* Input Area */
.mtchat__input-area {
  padding: 12px 16px;
  border-top: 1px solid var(--mtchat-border);
  background: var(--mtchat-bg);
}

.mtchat__input-form {
  display: flex;
  gap: 8px;
  align-items: center;
}

.mtchat__file-input {
  display: none;
}

.mtchat__btn--attach {
  width: 44px;
  height: 44px;
  padding: 0;
  border-radius: 50%;
  background: transparent;
  color: var(--mtchat-text-secondary);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.mtchat__btn--attach:hover:not(:disabled) {
  background: var(--mtchat-bg-secondary);
  color: var(--mtchat-primary);
}

.mtchat__input {
  flex: 1;
  padding: 10px 14px;
  border: 1px solid var(--mtchat-border);
  border-radius: 20px;
  font-size: 14px;
  outline: none;
  background: var(--mtchat-bg);
  color: var(--mtchat-text);
}

.mtchat__input:focus {
  border-color: var(--mtchat-primary);
}

.mtchat__join-prompt {
  text-align: center;
  padding: 16px;
  color: var(--mtchat-text-secondary);
}

.mtchat__join-prompt p {
  margin: 0 0 12px;
}

/* Buttons */
.mtchat__btn {
  padding: 8px 16px;
  border: none;
  border-radius: 20px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}

.mtchat__btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.mtchat__btn--primary {
  background: var(--mtchat-primary);
  color: white;
}

.mtchat__btn--primary:hover:not(:disabled) {
  background: var(--mtchat-primary-hover);
}

.mtchat__btn--secondary {
  background: var(--mtchat-bg-secondary);
  color: var(--mtchat-text);
}

.mtchat__btn--secondary:hover:not(:disabled) {
  background: var(--mtchat-border);
}

.mtchat__btn--send {
  background: var(--mtchat-primary);
  color: white;
  padding: 10px 20px;
}

.mtchat__btn--send:hover:not(:disabled) {
  background: var(--mtchat-primary-hover);
}

/* Empty States */
.mtchat__empty,
.mtchat__no-dialog {
  display: flex;
  align-items: center;
  justify-content: center;
  flex: 1;
  color: var(--mtchat-text-secondary);
  padding: 24px;
  text-align: center;
}

/* Loading */
.mtchat__loading {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 24px;
  color: var(--mtchat-text-secondary);
}

/* Info Panel - side column */
.mtchat__info-panel {
  width: 300px;
  min-width: 280px;
  flex-shrink: 0;
  border-left: 1px solid var(--mtchat-border);
  background: var(--mtchat-bg);
  overflow: hidden;
}

/* In inline mode, overlay the entire chat */
.mtchat__info-panel--inline {
  position: absolute;
  inset: 0;
  width: 100%;
  border-left: none;
  z-index: 50;
}

/* Info Panel Transition */
.mtchat-info-panel-enter-active,
.mtchat-info-panel-leave-active {
  transition: width 0.25s ease, opacity 0.25s ease;
}

.mtchat-info-panel-enter-from,
.mtchat-info-panel-leave-to {
  width: 0;
  min-width: 0;
  opacity: 0;
}

/* Inline mode transition - fade instead of width */
.mtchat__info-panel--inline.mtchat-info-panel-enter-from,
.mtchat__info-panel--inline.mtchat-info-panel-leave-to {
  width: 100%;
  opacity: 0;
}

/* Join Required State */
.mtchat__join-required {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 24px;
}

.mtchat__join-required-content {
  text-align: center;
  color: var(--mtchat-text-secondary);
}

.mtchat__join-required-content svg {
  color: var(--mtchat-text-secondary);
  margin-bottom: 16px;
  opacity: 0.6;
}

.mtchat__join-required-content p {
  margin: 0 0 16px;
  font-size: 15px;
  color: var(--mtchat-text-secondary);
}
</style>
