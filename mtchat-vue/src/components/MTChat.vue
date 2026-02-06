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
import type { MTChatConfig, ChatMode, DialogListItem, Message, Attachment, SystemMessageContent } from '../types'
import AttachmentPreview from './chat/AttachmentPreview.vue'
import AttachmentList from './chat/AttachmentList.vue'
import FileViewer from './chat/FileViewer.vue'
import ChatInfoPanel from './chat/ChatInfoPanel.vue'
import JoinDialog from './chat/JoinDialog.vue'
import MessageEditor from './chat/MessageEditor.vue'

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
const messagesContainer = ref<HTMLElement | null>(null)
const fileInputRef = ref<HTMLInputElement | null>(null)
const messageEditorRef = ref<InstanceType<typeof MessageEditor> | null>(null)
const editorIsEmpty = ref(true)
const searchInputRef = ref<HTMLInputElement | null>(null)
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

// Archived accordion state
const showArchivedAccordion = ref(false)

// Responsive state
const windowWidth = ref(typeof window !== 'undefined' ? window.innerWidth : 1200)
const windowHeight = ref(typeof window !== 'undefined' ? window.innerHeight : 800)

// Breakpoint detection
const isMobile = computed(() => windowWidth.value < 768)
const isTablet = computed(() => windowWidth.value >= 768 && windowWidth.value < 1200)
const isDesktop = computed(() => windowWidth.value >= 1200)

// Mobile view state: which column is currently visible
type MobileView = 'list' | 'chat' | 'info'
const mobileView = ref<MobileView>('list')

// Column resizer state (desktop only)
const sidebarWidth = ref(280)
const infoWidth = ref(300)
const isResizingSidebar = ref(false)
const isResizingInfo = ref(false)
const containerRef = ref<HTMLElement | null>(null)

// Resize constraints
const SIDEBAR_MIN_WIDTH = 200
const INFO_MIN_WIDTH = 240
const MAX_COLUMN_PERCENT = 30
const MAIN_MIN_PERCENT = 50

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

const hasArchivedDialogs = computed(() =>
  chat.archivedDialogs.value.length > 0
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

// Watch for info panel changes on mobile/tablet
watch(showInfoPanel, (show) => {
  if (isMobile.value || isTablet.value) {
    if (show) {
      mobileView.value = 'info'
    } else if (mobileView.value === 'info') {
      // When closing info panel (via X button), return to chat view
      mobileView.value = 'chat'
    }
  }
})

// Reset mobile view when switching to desktop
watch(isDesktop, (desktop) => {
  if (desktop) {
    mobileView.value = 'list'
  }
})

// Auto-scroll on new messages and mark as read
watch(
  () => chat.messages.value.length,
  async (newLength, oldLength) => {
    await nextTick()
    scrollToBottom()

    // Mark as read after initial load (not on new incoming messages)
    // Only if we have unread messages and just loaded the first batch
    if (oldLength === 0 && newLength > 0 && chat.firstUnreadMessageId.value) {
      // Delay to allow user to see the chat
      setTimeout(() => {
        chat.markAsRead()
      }, 1500)
    }
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

/**
 * Handle message submission from editor
 */
async function handleEditorSubmit(htmlContent: string) {
  if (!canSendMessage.value) return

  const attachments = fileUpload.getUploadedAttachments()

  // Clear editor and attachments
  messageEditorRef.value?.clear()
  fileUpload.clearAll()

  try {
    const message = await chat.sendMessage(
      htmlContent,
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

async function handleToggleArchive() {
  if (!chat.currentDialog.value) return

  const dialogId = chat.currentDialog.value.id

  try {
    if (chat.currentDialog.value.is_archived) {
      await chat.unarchiveDialog(dialogId)
    } else {
      await chat.archiveDialog(dialogId)
    }
  } catch (e) {
    // Error handled in composable
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

/**
 * Extract plain text from HTML content
 */
function stripHtml(html: string): string {
  // Create a temporary element to parse HTML
  if (typeof document !== 'undefined') {
    const tmp = document.createElement('div')
    tmp.innerHTML = html
    return tmp.textContent || tmp.innerText || ''
  }
  // Fallback for SSR - simple regex strip
  return html.replace(/<[^>]*>/g, '')
}

function getQuotedText(messageId: string): string {
  const msg = chat.messages.value.find((m) => m.id === messageId)
  if (!msg) return t.value.chat.messageDeleted
  const plainText = stripHtml(msg.content)
  return truncateText(plainText, 60)
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
  if (!msg.sender_id) return '' // System message
  return getSenderDisplayName(msg.sender_id)
}

/**
 * Format system message content from JSON to localized string
 */
function formatSystemMessage(message: Message): string {
  if (message.message_type !== 'system') return message.content

  try {
    const data: SystemMessageContent = JSON.parse(message.content)

    switch (data.event) {
      case 'chat_created': {
        const participants = data.participants
          ?.map(p => p.company ? `${p.name} (${p.company})` : p.name)
          .join(', ') || ''
        return t.value.system.chatCreated.replace('{participants}', participants)
      }
      case 'participant_joined': {
        const name = data.company
          ? `${data.name} (${data.company})`
          : data.name || ''
        return t.value.system.participantJoined.replace('{name}', name)
      }
      case 'participant_left': {
        return t.value.system.participantLeft.replace('{name}', data.name || '')
      }
      default:
        return message.content
    }
  } catch {
    return message.content // Fallback for invalid JSON
  }
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

// Window resize handler
function handleWindowResize() {
  windowWidth.value = window.innerWidth
  windowHeight.value = window.innerHeight
}

// Mobile/inline navigation handlers
function goBack() {
  // In inline mode, just close the info panel
  if (isInlineMode.value && showInfoPanel.value) {
    showInfoPanel.value = false
    return
  }

  // Mobile/tablet navigation
  if (mobileView.value === 'info') {
    mobileView.value = 'chat'
    showInfoPanel.value = false
  } else if (mobileView.value === 'chat') {
    mobileView.value = 'list'
  }
}

// Override selectDialog to handle mobile navigation
const originalSelectDialog = handleSelectDialog
async function handleSelectDialogResponsive(dialog: DialogListItem) {
  await originalSelectDialog(dialog)
  if (isMobile.value || isTablet.value) {
    mobileView.value = 'chat'
  }
}

// Column resize handlers
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

  // Ensure main area has at least 50%
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

  // Ensure main area has at least 50%
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

// Keyboard handler for Esc and Cmd+K/Ctrl+K
function handleKeydown(e: KeyboardEvent) {
  // Cmd+K or Ctrl+K - focus search input
  if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
    e.preventDefault()
    if (searchInputRef.value && !isInlineMode.value && props.showSidebar) {
      searchInputRef.value.focus()
      searchInputRef.value.select()
    }
    return
  }

  // Esc - clear reply
  if (e.key === 'Escape' && chat.replyToMessage.value) {
    chat.clearReplyTo()
  }
}

// Lifecycle - keyboard handlers and window resize
onMounted(() => {
  document.addEventListener('keydown', handleKeydown)
  window.addEventListener('resize', handleWindowResize)
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
  window.removeEventListener('resize', handleWindowResize)
  stopResize()
})

// Expose for parent access
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
    <!-- Sidebar (Full mode only) -->
    <aside
      v-if="!isInlineMode && showSidebar"
      class="mtchat__sidebar"
      :style="isDesktop ? { width: `${sidebarWidth}px` } : undefined"
    >
      <!-- Search -->
      <div class="mtchat__search">
        <input
          ref="searchInputRef"
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

      <!-- Dialog List Container -->
      <div class="mtchat__dialog-list-container">
        <!-- Scrollable Dialog List -->
        <div class="mtchat__dialog-list">
          <div
            v-for="dialog in currentDialogsList"
            :key="dialog.id"
            :class="['mtchat__dialog-item', { 'mtchat__dialog-item--active': chat.currentDialog.value?.id === dialog.id }]"
            @click="handleSelectDialogResponsive(dialog)"
          >
            <div class="mtchat__dialog-title">
              {{ dialog.title || `${dialog.object_type}/${dialog.object_id}` }}
            </div>
            <div class="mtchat__dialog-meta">
              <span class="mtchat__dialog-participants">
                {{ tt('chat.participants', { count: dialog.participants_count }) }}
              </span>
            </div>
            <span v-if="dialog.unread_count && dialog.unread_count > 0" class="mtchat__unread-badge">
              {{ dialog.unread_count > 99 ? '99+' : dialog.unread_count }}
            </span>
          </div>

          <div v-if="currentDialogsList.length === 0 && !hasArchivedDialogs" class="mtchat__empty">
            {{ searchInput
              ? t.search.noResults
              : (activeTab === 'participating' ? t.chat.noActiveChats : t.chat.noAvailableChats)
            }}
          </div>
        </div>

        <!-- Archived Accordion (sticky at bottom, only in participating tab) -->
        <div
          v-if="activeTab === 'participating' && hasArchivedDialogs"
          :class="['mtchat__archived-section', { 'mtchat__archived-section--open': showArchivedAccordion }]"
        >
          <button
            class="mtchat__archived-toggle"
            @click="showArchivedAccordion = !showArchivedAccordion"
          >
            <svg
              width="12"
              height="12"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <polyline points="9 18 15 12 9 6"/>
            </svg>
            {{ t.chat.archived }} ({{ chat.archivedDialogs.value.length }})
          </button>

          <div v-if="showArchivedAccordion" class="mtchat__archived-list">
            <div
              v-for="dialog in chat.archivedDialogs.value"
              :key="dialog.id"
              :class="[
                'mtchat__dialog-item',
                'mtchat__dialog-item--archived',
                { 'mtchat__dialog-item--active': chat.currentDialog.value?.id === dialog.id }
              ]"
              @click="handleSelectDialogResponsive(dialog)"
            >
              <div class="mtchat__dialog-title">
                {{ dialog.title || `${dialog.object_type}/${dialog.object_id}` }}
              </div>
              <div class="mtchat__dialog-meta">
                <span class="mtchat__dialog-participants">
                  {{ tt('chat.participants', { count: dialog.participants_count }) }}
                </span>
              </div>
              <span v-if="dialog.unread_count && dialog.unread_count > 0" class="mtchat__unread-badge">
                {{ dialog.unread_count > 99 ? '99+' : dialog.unread_count }}
              </span>
            </div>
          </div>
        </div>
      </div>
    </aside>

    <!-- Sidebar Resizer (Desktop only) -->
    <div
      v-if="!isInlineMode && showSidebar && isDesktop"
      class="mtchat__resizer"
      @mousedown="startSidebarResize"
      @touchstart="startSidebarResize"
    ></div>

    <!-- Main Chat Area -->
    <main class="mtchat__main">
      <!-- Header -->
      <header v-if="showHeader && hasDialog" class="mtchat__header">
        <!-- Back button (mobile/tablet/inline with info open) -->
        <button
          v-if="(isMobile && mobileView === 'chat') || (isTablet && showInfoPanel) || (isInlineMode && showInfoPanel)"
          class="mtchat__back-btn"
          :title="t.tooltips.back"
          @click="goBack"
        >
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="15 18 9 12 15 6"/>
          </svg>
        </button>

        <button
          class="mtchat__header-info"
          @click="showInfoPanel = true"
          :title="t.tooltips.chatInfo"
        >
          <div class="mtchat__header-title-row">
            <h2 class="mtchat__header-title">{{ dialogTitle }}</h2>
            <span v-if="chat.currentDialog.value?.is_archived" class="mtchat__archived-badge">
              {{ t.chat.archived }}
            </span>
          </div>
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
                class="mtchat__menu-item"
                @click="handleToggleArchive(); showHeaderMenu = false"
                :disabled="chat.isLoading.value"
              >
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <polyline points="21 8 21 21 3 21 3 8"/>
                  <rect x="1" y="3" width="22" height="5"/>
                  <line x1="10" y1="12" x2="14" y2="12"/>
                </svg>
                {{ chat.currentDialog.value?.is_archived ? t.buttons.unarchive : t.buttons.archive }}
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

          <!-- SYSTEM MESSAGE -->
          <div
            v-if="message.message_type === 'system'"
            :data-message-id="message.id"
            class="mtchat__system-message"
          >
            {{ formatSystemMessage(message) }}
          </div>

          <!-- USER MESSAGE -->
          <div
            v-else
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
                {{ message.sender_id ? getSenderDisplayName(message.sender_id) : '' }}
              </span>
              <span class="mtchat__message-time">{{ formatTime(message.sent_at) }}</span>
            </div>

            <!-- Content (HTML or plain text) -->
            <div
              v-if="message.content"
              class="mtchat__message-content"
              v-html="message.content"
            ></div>

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
                {{ chat.replyToMessage.value.sender_id ? getSenderDisplayName(chat.replyToMessage.value.sender_id) : '' }}
              </div>
              <div class="mtchat__reply-text">
                {{ truncateText(stripHtml(chat.replyToMessage.value.content), 100) }}
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

          <!-- Hidden file input -->
          <input
            ref="fileInputRef"
            type="file"
            multiple
            class="mtchat__file-input"
            @change="handleFileChange"
          />

          <!-- Message Editor -->
          <MessageEditor
            ref="messageEditorRef"
            :placeholder="t.input.placeholder"
            :disabled="chat.isLoading.value"
            :participants="chat.participants.value"
            :current-user-id="config.userId"
            @submit="handleEditorSubmit"
            @update:is-empty="editorIsEmpty = $event"
            @attach="handleFileSelect"
          />
        </template>
      </div>
    </main>

    <!-- Info Panel Resizer (Desktop only) -->
    <div
      v-if="showInfoPanel && hasDialog && isDesktop && !isInlineMode"
      class="mtchat__resizer"
      @mousedown="startInfoResize"
      @touchstart="startInfoResize"
    ></div>

    <!-- Info Panel (side column in full mode, overlay in inline mode) -->
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
  /* === LAYOUT TOKENS === */
  --mtchat-sidebar-width: 280px;
  --mtchat-sidebar-min-width: 200px;
  --mtchat-sidebar-max-percent: 30;

  --mtchat-info-width: 300px;
  --mtchat-info-min-width: 240px;
  --mtchat-info-max-percent: 30;

  --mtchat-main-min-percent: 50;

  --mtchat-header-height: 48px;
  --mtchat-resizer-width: 4px;

  /* === SPACING TOKENS === */
  --mtchat-spacing-xs: 4px;
  --mtchat-spacing-sm: 8px;
  --mtchat-spacing-md: 12px;
  --mtchat-spacing-lg: 16px;
  --mtchat-spacing-xl: 24px;

  /* === SIZING TOKENS === */
  --mtchat-button-size: 44px;
  --mtchat-icon-size: 20px;
  --mtchat-avatar-size: 40px;
  --mtchat-input-height: 44px;

  /* === BORDER TOKENS === */
  --mtchat-border-radius-sm: 4px;
  --mtchat-border-radius-md: 8px;
  --mtchat-border-radius-lg: 12px;
  --mtchat-border-radius-full: 50%;

  /* === TRANSITION TOKENS === */
  --mtchat-transition-fast: 150ms ease;
  --mtchat-transition-normal: 300ms ease;

  /* === Z-INDEX TOKENS === */
  --mtchat-z-base: 1;
  --mtchat-z-overlay: 10;
  --mtchat-z-modal: 100;

  display: flex;
  height: 100%;
  min-height: 400px;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  font-size: 14px;
  border: 1px solid var(--mtchat-border, #e0e0e0);
  border-radius: var(--mtchat-border-radius-md);
  overflow: hidden;
  position: relative;
}

.mtchat--inline {
  border-radius: var(--mtchat-border-radius-sm);
  min-height: 0;
  border: none;
}

/* Disable text selection while resizing */
.mtchat--resizing {
  user-select: none;
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
  width: var(--mtchat-sidebar-width);
  min-width: var(--mtchat-sidebar-min-width);
  border-right: 1px solid var(--mtchat-border);
  background: var(--mtchat-bg);
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
  transition: transform var(--mtchat-transition-normal);
}

/* Column Resizer */
.mtchat__resizer {
  width: var(--mtchat-resizer-width);
  background: transparent;
  cursor: col-resize;
  flex-shrink: 0;
  position: relative;
  z-index: var(--mtchat-z-base);
  transition: background var(--mtchat-transition-fast);
}

.mtchat__resizer:hover,
.mtchat--resizing .mtchat__resizer {
  background: var(--mtchat-primary);
}

/* Search */
.mtchat__search {
  height: var(--mtchat-header-height);
  padding: 0 var(--mtchat-spacing-md);
  border-bottom: 1px solid var(--mtchat-border);
  display: flex;
  align-items: center;
  gap: var(--mtchat-spacing-sm);
  position: relative;
  flex-shrink: 0;
}

.mtchat__search-input {
  flex: 1;
  height: 36px;
  padding: 0 var(--mtchat-spacing-md);
  padding-right: 32px;
  border: 1px solid var(--mtchat-border);
  border-radius: var(--mtchat-border-radius-md);
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
  right: calc(var(--mtchat-spacing-md) + var(--mtchat-spacing-sm));
  top: 50%;
  transform: translateY(-50%);
  background: none;
  border: none;
  color: var(--mtchat-text-secondary);
  cursor: pointer;
  padding: var(--mtchat-spacing-xs);
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--mtchat-border-radius-sm);
}

.mtchat__search-clear:hover {
  color: var(--mtchat-text);
  background: var(--mtchat-bg-hover);
}

/* Tabs */
.mtchat__tabs {
  height: var(--mtchat-header-height);
  display: flex;
  border-bottom: 1px solid var(--mtchat-border);
  flex-shrink: 0;
}

.mtchat__tab {
  flex: 1;
  padding: 0 var(--mtchat-spacing-sm);
  background: none;
  border: none;
  border-bottom: 2px solid transparent;
  color: var(--mtchat-text-secondary);
  cursor: pointer;
  font-size: 13px;
  font-weight: 500;
  transition: all var(--mtchat-transition-fast);
  display: flex;
  align-items: center;
  justify-content: center;
  gap: var(--mtchat-spacing-xs);
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
  padding: 2px 6px;
  background: var(--mtchat-bg-secondary);
  border-radius: var(--mtchat-border-radius-lg);
  font-size: 11px;
}

/* Dialog List Container */
.mtchat__dialog-list-container {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
  overflow: hidden;
}

/* Dialog List */
.mtchat__dialog-list {
  flex: 1;
  overflow-y: auto;
  min-height: 0;
}

.mtchat__dialog-item {
  padding: var(--mtchat-spacing-sm) var(--mtchat-spacing-md);
  border-bottom: 1px solid var(--mtchat-border);
  cursor: pointer;
  transition: background var(--mtchat-transition-fast);
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
  font-size: 13px;
  color: var(--mtchat-text);
  margin-bottom: 2px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.mtchat__dialog-meta {
  font-size: 11px;
  color: var(--mtchat-text-secondary);
  display: flex;
  gap: var(--mtchat-spacing-sm);
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
  height: var(--mtchat-header-height);
  padding: 0 var(--mtchat-spacing-lg);
  border-bottom: 1px solid var(--mtchat-border);
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--mtchat-spacing-sm);
  background: var(--mtchat-bg);
  flex-shrink: 0;
}

/* Back button */
.mtchat__back-btn {
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  background: none;
  border-radius: var(--mtchat-border-radius-md);
  cursor: pointer;
  color: var(--mtchat-text-secondary);
  transition: background var(--mtchat-transition-fast), color var(--mtchat-transition-fast);
  flex-shrink: 0;
}

.mtchat__back-btn:hover {
  background: var(--mtchat-bg-hover);
  color: var(--mtchat-text);
}

.mtchat__header-title-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.mtchat__header-title {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  color: var(--mtchat-text);
}

.mtchat__archived-badge {
  font-size: 11px;
  font-weight: 500;
  padding: 2px 8px;
  background: var(--mtchat-text-secondary);
  color: var(--mtchat-bg);
  border-radius: 4px;
  text-transform: uppercase;
  letter-spacing: 0.3px;
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

/* System message (centered, gray text) */
.mtchat__system-message {
  text-align: center;
  color: var(--mtchat-text-secondary);
  font-size: 13px;
  padding: 8px 16px;
  margin: 4px 0;
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
  word-wrap: break-word;
}

/* HTML content styles in messages */
.mtchat__message-content :deep(p) {
  margin: 0;
}

.mtchat__message-content :deep(p + p) {
  margin-top: 8px;
}

.mtchat__message-content :deep(strong) {
  font-weight: 600;
}

.mtchat__message-content :deep(em) {
  font-style: italic;
}

.mtchat__message-content :deep(u) {
  text-decoration: underline;
}

.mtchat__message-content :deep(s) {
  text-decoration: line-through;
}

.mtchat__message-content :deep(code) {
  font-family: 'SF Mono', Monaco, Menlo, monospace;
  font-size: 13px;
  background: var(--mtchat-bg-secondary);
  padding: 2px 6px;
  border-radius: 4px;
}

.mtchat__message-content :deep(pre) {
  font-family: 'SF Mono', Monaco, Menlo, monospace;
  font-size: 13px;
  background: var(--mtchat-bg-secondary);
  padding: 12px 16px;
  border-radius: 6px;
  margin: 8px 0;
  overflow-x: auto;
  white-space: pre-wrap;
}

.mtchat__message-content :deep(pre code) {
  background: none;
  padding: 0;
}

.mtchat__message-content :deep(blockquote) {
  border-left: 3px solid var(--mtchat-primary);
  padding-left: 12px;
  margin: 8px 0;
  color: var(--mtchat-text-secondary);
}

.mtchat__message-content :deep(ul),
.mtchat__message-content :deep(ol) {
  padding-left: 24px;
  margin: 8px 0;
}

.mtchat__message-content :deep(li) {
  margin: 4px 0;
}

.mtchat__message-content :deep(a) {
  color: var(--mtchat-primary);
  text-decoration: underline;
}

.mtchat__message-content :deep(a:hover) {
  text-decoration: none;
}

.mtchat__message-content :deep(.mtchat-mention) {
  color: var(--mtchat-primary);
  background: rgba(59, 130, 246, 0.1);
  padding: 2px 4px;
  border-radius: 4px;
  font-weight: 500;
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
  padding: var(--mtchat-spacing-md) var(--mtchat-spacing-lg);
  border-top: 1px solid var(--mtchat-border);
  background: var(--mtchat-bg);
  flex-shrink: 0;
}

.mtchat__file-input {
  display: none;
}

.mtchat__join-prompt {
  text-align: center;
  padding: var(--mtchat-spacing-lg);
  color: var(--mtchat-text-secondary);
}

.mtchat__join-prompt p {
  margin: 0 0 var(--mtchat-spacing-md);
}

/* Buttons */
.mtchat__btn {
  padding: var(--mtchat-spacing-sm) var(--mtchat-spacing-lg);
  border: none;
  border-radius: calc(var(--mtchat-input-height) / 2);
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all var(--mtchat-transition-fast);
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
  width: var(--mtchat-button-size);
  height: var(--mtchat-button-size);
  padding: 0;
  border-radius: var(--mtchat-border-radius-full);
  background: var(--mtchat-primary);
  color: white;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.mtchat__btn--send:hover:not(:disabled) {
  background: var(--mtchat-primary-hover);
}

.mtchat__btn--send svg {
  width: var(--mtchat-icon-size);
  height: var(--mtchat-icon-size);
}

/* Empty States */
.mtchat__empty,
.mtchat__no-dialog {
  display: flex;
  align-items: center;
  justify-content: center;
  flex: 1;
  color: var(--mtchat-text-secondary);
  padding: var(--mtchat-spacing-xl);
  text-align: center;
}

/* Loading */
.mtchat__loading {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: var(--mtchat-spacing-xl);
  color: var(--mtchat-text-secondary);
}

/* Info Panel - side column */
.mtchat__info-panel {
  width: var(--mtchat-info-width);
  min-width: var(--mtchat-info-min-width);
  flex-shrink: 0;
  border-left: 1px solid var(--mtchat-border);
  background: var(--mtchat-bg);
  overflow: hidden;
  transition: transform var(--mtchat-transition-normal);
}

/* In inline mode, overlay the entire chat */
.mtchat__info-panel--inline {
  position: absolute;
  inset: 0;
  width: 100%;
  border-left: none;
  z-index: var(--mtchat-z-modal);
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

/* Archived Section (sticky at bottom) */
.mtchat__archived-section {
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  background: var(--mtchat-bg-secondary);
  border-top: 2px solid var(--mtchat-border);
  box-shadow: 0 -2px 8px rgba(0, 0, 0, 0.05);
}

/* When open: exactly 50% height */
.mtchat__archived-section--open {
  flex: 1;
  max-height: 50%;
  min-height: 50%;
}

.mtchat__archived-toggle {
  width: 100%;
  padding: 10px 16px;
  display: flex;
  align-items: center;
  gap: 8px;
  background: var(--mtchat-border);
  border: none;
  color: var(--mtchat-text-secondary);
  cursor: pointer;
  font-size: 12px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  text-align: left;
  flex-shrink: 0;
}

.mtchat__archived-toggle:hover {
  background: var(--mtchat-bg-hover);
  color: var(--mtchat-text);
}

.mtchat--dark .mtchat__archived-toggle {
  background: var(--mtchat-bg-hover);
}

.mtchat--dark .mtchat__archived-toggle:hover {
  background: var(--mtchat-border);
}

.mtchat__archived-toggle svg {
  transition: transform 0.2s;
  opacity: 0.6;
}

.mtchat__archived-section--open .mtchat__archived-toggle svg {
  transform: rotate(90deg);
}

.mtchat__archived-list {
  flex: 1;
  overflow-y: auto;
  min-height: 0;
  background: var(--mtchat-bg);
}

.mtchat__dialog-item--archived {
  opacity: 0.7;
}

.mtchat__dialog-item--archived:hover {
  opacity: 1;
}

/* =====================================================
   RESPONSIVE LAYOUTS
   ===================================================== */

/* Mobile Layout (< 768px) */
@media (max-width: 767px) {
  .mtchat--mobile {
    position: relative;
  }

  .mtchat--mobile .mtchat__sidebar,
  .mtchat--mobile .mtchat__main,
  .mtchat--mobile .mtchat__info-panel {
    position: absolute;
    inset: 0;
    width: 100% !important;
    min-width: 0 !important;
    border: none;
  }

  /* Hide resizers on mobile */
  .mtchat--mobile .mtchat__resizer {
    display: none;
  }

  /* Sidebar (list view) */
  .mtchat--mobile .mtchat__sidebar {
    z-index: 1;
    transform: translateX(0);
  }

  .mtchat--mobile.mtchat--view-chat .mtchat__sidebar,
  .mtchat--mobile.mtchat--view-info .mtchat__sidebar {
    transform: translateX(-100%);
  }

  /* Main (chat view) */
  .mtchat--mobile .mtchat__main {
    z-index: 2;
    transform: translateX(100%);
  }

  .mtchat--mobile.mtchat--view-chat .mtchat__main,
  .mtchat--mobile.mtchat--view-info .mtchat__main {
    transform: translateX(0);
  }

  /* Info Panel (info view) */
  .mtchat--mobile .mtchat__info-panel {
    z-index: 3;
    transform: translateX(100%);
  }

  .mtchat--mobile.mtchat--view-info .mtchat__info-panel {
    transform: translateX(0);
  }
}

/* Tablet Layout (768px - 1199px) */
@media (min-width: 768px) and (max-width: 1199px) {
  .mtchat--tablet {
    position: relative;
  }

  /* Hide resizers on tablet */
  .mtchat--tablet .mtchat__resizer {
    display: none;
  }

  /* Default: sidebar + main */
  .mtchat--tablet .mtchat__sidebar {
    width: var(--mtchat-sidebar-width) !important;
    transition: transform var(--mtchat-transition-normal), opacity var(--mtchat-transition-normal);
  }

  .mtchat--tablet .mtchat__main {
    flex: 1;
  }

  /* When info panel is open: hide sidebar, show main + info */
  .mtchat--tablet.mtchat--view-info .mtchat__sidebar {
    position: absolute;
    left: 0;
    top: 0;
    bottom: 0;
    transform: translateX(-100%);
    opacity: 0;
    pointer-events: none;
  }

  .mtchat--tablet .mtchat__info-panel {
    width: var(--mtchat-info-width) !important;
    min-width: var(--mtchat-info-min-width) !important;
    transition: transform var(--mtchat-transition-normal);
  }

  /* Info panel slides in from right */
  .mtchat--tablet .mtchat__info-panel:not(.mtchat__info-panel--inline) {
    position: relative;
  }
}

/* Desktop Layout (≥ 1200px) - all three columns visible */
@media (min-width: 1200px) {
  .mtchat--desktop .mtchat__sidebar {
    flex-shrink: 0;
  }

  .mtchat--desktop .mtchat__main {
    flex: 1;
    min-width: 0;
  }

  .mtchat--desktop .mtchat__info-panel {
    flex-shrink: 0;
  }
}

/* Small height - archived accordion takes full height */
@media (max-height: 599px) {
  .mtchat__archived-section--open {
    max-height: none;
    min-height: 0;
  }
}
</style>
