<script setup lang="ts">
/**
 * MTChat Vue Component
 *
 * Two display modes:
 * - Full mode: Dialog list with tabs (My/Available) + chat area
 * - Inline mode: Single chat for a business object
 */

import { computed, ref, watch, nextTick } from 'vue'
import { useChat } from '../composables/useChat'
import { useFileUpload } from '../composables/useFileUpload'
import type { MTChatConfig, ChatMode, DialogListItem, Message, Attachment } from '../types'
import AttachmentPreview from './chat/AttachmentPreview.vue'
import AttachmentList from './chat/AttachmentList.vue'
import FileViewer from './chat/FileViewer.vue'

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

// File viewer state
const showFileViewer = ref(false)
const fileViewerIndex = ref(0)

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
  hasDialog.value &&
  (chat.currentDialog.value?.i_am_participant || isInlineMode.value)
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

async function handleJoinDialog() {
  if (!chat.currentDialog.value) return

  try {
    await chat.joinDialog(chat.currentDialog.value.id)
    emit('dialog-joined', chat.currentDialog.value.id)
  } catch (e) {
    // Error already handled
  }
}

async function handleLeaveDialog() {
  if (!chat.currentDialog.value) return

  const dialogId = chat.currentDialog.value.id
  try {
    await chat.leaveDialog(dialogId)
    emit('dialog-left', dialogId)
  } catch (e) {
    // Error already handled
  }
}

function formatTime(dateString: string): string {
  const date = new Date(dateString)
  return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
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
      <!-- Tabs -->
      <div class="mtchat__tabs">
        <button
          :class="['mtchat__tab', { 'mtchat__tab--active': activeTab === 'participating' }]"
          @click="activeTab = 'participating'"
        >
          My Chats
          <span v-if="chat.participatingDialogs.value.length" class="mtchat__tab-count">
            {{ chat.participatingDialogs.value.length }}
          </span>
        </button>
        <button
          :class="['mtchat__tab', { 'mtchat__tab--active': activeTab === 'available' }]"
          @click="activeTab = 'available'"
        >
          Available
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
              {{ dialog.participants_count }} participants
            </span>
          </div>
          <div v-if="dialog.can_join && !dialog.i_am_participant" class="mtchat__dialog-badge">
            Can Join
          </div>
        </div>

        <div v-if="currentDialogsList.length === 0" class="mtchat__empty">
          {{ activeTab === 'participating' ? 'No active chats' : 'No available chats' }}
        </div>
      </div>
    </aside>

    <!-- Main Chat Area -->
    <main class="mtchat__main">
      <!-- Header -->
      <header v-if="showHeader && hasDialog" class="mtchat__header">
        <div class="mtchat__header-info">
          <h2 class="mtchat__header-title">{{ dialogTitle }}</h2>
          <div class="mtchat__header-meta">
            <span class="mtchat__header-participants">
              {{ chat.currentDialog.value?.participants_count || 0 }} participants
            </span>
            <span :class="['mtchat__status', { 'mtchat__status--connected': chat.isConnected.value }]">
              {{ chat.isConnected.value ? 'Connected' : 'Disconnected' }}
            </span>
          </div>
        </div>
        <div class="mtchat__header-actions">
          <button
            v-if="canJoin"
            class="mtchat__btn mtchat__btn--primary"
            @click="handleJoinDialog"
            :disabled="chat.isLoading.value"
          >
            Join Chat
          </button>
          <button
            v-else-if="chat.currentDialog.value?.i_am_participant"
            class="mtchat__btn mtchat__btn--secondary"
            @click="handleLeaveDialog"
            :disabled="chat.isLoading.value"
          >
            Leave
          </button>
        </div>
      </header>

      <!-- No Dialog Selected -->
      <div v-if="!hasDialog" class="mtchat__no-dialog">
        <p v-if="isInlineMode">No chat available for this object</p>
        <p v-else>Select a chat to start messaging</p>
      </div>

      <!-- Messages -->
      <div v-else ref="messagesContainer" class="mtchat__messages">
        <div
          v-for="message in chat.messages.value"
          :key="message.id"
          :class="['mtchat__message', { 'mtchat__message--own': message.sender_id === config.userId }]"
        >
          <div class="mtchat__message-sender">
            {{ message.sender_id === config.userId ? 'You' : message.sender_id.slice(0, 8) }}
          </div>
          <div v-if="message.content" class="mtchat__message-content">{{ message.content }}</div>
          <AttachmentList
            v-if="message.attachments && message.attachments.length > 0"
            :attachments="message.attachments"
            @open-gallery="(index) => openGalleryAtIndex(message, index)"
            @open-file="openFileViewer"
          />
          <div class="mtchat__message-time">{{ formatTime(message.sent_at) }}</div>
        </div>

        <div v-if="chat.messages.value.length === 0" class="mtchat__empty">
          No messages yet
        </div>
      </div>

      <!-- Input Area -->
      <div v-if="hasDialog" class="mtchat__input-area">
        <div v-if="!canSendMessage && canJoin" class="mtchat__join-prompt">
          <p>Join this chat to send messages</p>
          <button class="mtchat__btn mtchat__btn--primary" @click="handleJoinDialog">
            Join Chat
          </button>
        </div>
        <template v-else-if="canSendMessage">
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
              title="Attach files"
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
              placeholder="Type a message..."
              :disabled="chat.isLoading.value"
            />

            <!-- Send button -->
            <button
              type="submit"
              class="mtchat__btn mtchat__btn--send"
              :disabled="!hasContentToSend || chat.isLoading.value || fileUpload.isUploading.value"
            >
              Send
            </button>
          </form>
        </template>
      </div>
    </main>

    <!-- Unified File Viewer Modal -->
    <FileViewer
      :show="showFileViewer"
      :files="allAttachments"
      :initial-index="fileViewerIndex"
      @close="closeFileViewer"
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
}

.mtchat--inline {
  border-radius: 4px;
  min-height: 0;
  border: none;
}

/* Light theme (default) */
.mtchat--light {
  --mtchat-bg: #ffffff;
  --mtchat-bg-secondary: #f5f5f5;
  --mtchat-text: #333333;
  --mtchat-text-secondary: #666666;
  --mtchat-border: #e0e0e0;
  --mtchat-primary: #1976d2;
  --mtchat-primary-hover: #1565c0;
  --mtchat-message-own: #e3f2fd;
  --mtchat-message-other: #f5f5f5;
}

/* Dark theme */
.mtchat--dark {
  --mtchat-bg: #1e1e1e;
  --mtchat-bg-secondary: #2d2d2d;
  --mtchat-text: #ffffff;
  --mtchat-text-secondary: #b0b0b0;
  --mtchat-border: #404040;
  --mtchat-primary: #64b5f6;
  --mtchat-primary-hover: #42a5f5;
  --mtchat-message-own: #1565c0;
  --mtchat-message-other: #2d2d2d;
}

/* Sidebar */
.mtchat__sidebar {
  width: 280px;
  border-right: 1px solid var(--mtchat-border);
  background: var(--mtchat-bg);
  display: flex;
  flex-direction: column;
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

/* Main Area */
.mtchat__main {
  flex: 1;
  display: flex;
  flex-direction: column;
  background: var(--mtchat-bg);
  min-width: 0;
  min-height: 0;
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

/* Messages */
.mtchat__messages {
  flex: 1;
  overflow-y: auto;
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.mtchat__message {
  max-width: 70%;
  padding: 8px 12px;
  border-radius: 12px;
  background: var(--mtchat-message-other);
  align-self: flex-start;
}

.mtchat__message--own {
  background: var(--mtchat-message-own);
  align-self: flex-end;
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
</style>
