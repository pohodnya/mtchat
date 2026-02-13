<script setup lang="ts">
/**
 * ChatInput - Message input with reply/edit preview and attachments
 */

import { ref, computed, watch } from 'vue'
import type { Message, DialogParticipant, AttachmentInput } from '../../types'
import { useI18n } from '../../i18n'
import { useFileUpload } from '../../composables/useFileUpload'
import type { MTChatApi } from '../../sdk/api'
import AttachmentPreview from './AttachmentPreview.vue'
import MessageEditor from './MessageEditor.vue'
import Icon from '../Icon.vue'

const props = defineProps<{
  dialogId: string
  api: MTChatApi
  participants: DialogParticipant[]
  currentUserId: string
  replyToMessage: Message | null
  editingMessage: Message | null
  isLoading: boolean
  canSend: boolean
  canJoin: boolean
}>()

const emit = defineEmits<{
  send: [content: string, attachments?: AttachmentInput[]]
  edit: [messageId: string, content: string]
  cancelReply: []
  cancelEdit: []
  join: []
}>()

// i18n
const { t } = useI18n()

// Refs
const fileInputRef = ref<HTMLInputElement | null>(null)
const editorRef = ref<InstanceType<typeof MessageEditor> | null>(null)

// State
const editorIsEmpty = ref(true)

// File upload
const dialogIdRef = computed(() => props.dialogId)
const fileUpload = useFileUpload({
  dialogId: dialogIdRef,
  api: props.api,
})

// Helpers
function stripHtml(html: string): string {
  if (typeof document !== 'undefined') {
    const tmp = document.createElement('div')
    tmp.innerHTML = html
    return tmp.textContent || tmp.innerText || ''
  }
  return html.replace(/<[^>]*>/g, '')
}

function truncateText(text: string, maxLength: number): string {
  if (text.length <= maxLength) return text
  return text.slice(0, maxLength) + '...'
}

function getSenderDisplayName(senderId: string): string {
  const participant = props.participants.find(p => p.user_id === senderId)
  if (participant?.display_name) return participant.display_name
  return senderId === props.currentUserId ? t.value.user.you : senderId.slice(0, 8)
}

// Handle submit
async function handleSubmit(htmlContent: string) {
  if (!props.canSend) return

  if (props.editingMessage) {
    // Edit mode
    emit('edit', props.editingMessage.id, htmlContent)
    editorRef.value?.clear()
    return
  }

  // Send mode
  const attachments = fileUpload.getUploadedAttachments()
  editorRef.value?.clear()
  fileUpload.clearAll()

  emit('send', htmlContent, attachments.length > 0 ? attachments : undefined)
}

// File handling
function handleFileSelect() {
  fileInputRef.value?.click()
}

function handleFileChange(event: Event) {
  const input = event.target as HTMLInputElement
  if (input.files && input.files.length > 0) {
    fileUpload.addFiles(Array.from(input.files))
    input.value = ''
  }
}

// Arrow up to edit last message (handled by parent)
function handleArrowUp() {
  if (editorIsEmpty.value && !props.editingMessage) {
    // Parent should handle finding last own message
    // This is emitted to MTChat which has access to messages
  }
}

// Cancel edit
function cancelEdit() {
  emit('cancelEdit')
  editorRef.value?.clear()
}

// When entering edit mode, set content
watch(() => props.editingMessage, (message) => {
  if (message) {
    editorRef.value?.setContent(message.content)
    editorRef.value?.focus()
  }
})

// Expose
defineExpose({
  focus: () => editorRef.value?.focus(),
  clear: () => editorRef.value?.clear(),
  setContent: (content: string) => editorRef.value?.setContent(content),
})
</script>

<template>
  <div class="chat-input">
    <!-- Join prompt (not a participant) -->
    <div v-if="!canSend && canJoin" class="chat-input__join-prompt">
      <p>{{ t.chat.joinToSend }}</p>
      <button class="chat-input__btn chat-input__btn--primary" @click="emit('join')">
        {{ t.buttons.join }}
      </button>
    </div>

    <template v-else-if="canSend">
      <!-- Edit preview -->
      <div v-if="editingMessage" class="chat-input__preview chat-input__preview--edit">
        <div class="chat-input__preview-indicator"></div>
        <div class="chat-input__preview-content">
          <div class="chat-input__preview-label">{{ t.chat.editing }}</div>
          <div class="chat-input__preview-text">
            {{ truncateText(stripHtml(editingMessage.content), 100) }}
          </div>
        </div>
        <button class="chat-input__preview-cancel" @click="cancelEdit">
          <Icon name="close" :size="16" />
        </button>
      </div>

      <!-- Reply preview -->
      <div v-if="replyToMessage && !editingMessage" class="chat-input__preview chat-input__preview--reply">
        <div class="chat-input__preview-indicator"></div>
        <div class="chat-input__preview-content">
          <div class="chat-input__preview-author">
            {{ replyToMessage.sender_id ? getSenderDisplayName(replyToMessage.sender_id) : '' }}
          </div>
          <div class="chat-input__preview-text">
            {{ truncateText(stripHtml(replyToMessage.content), 100) }}
          </div>
        </div>
        <button class="chat-input__preview-cancel" @click="emit('cancelReply')">
          <Icon name="close" :size="16" />
        </button>
      </div>

      <!-- Attachment preview -->
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
        class="chat-input__file-input"
        @change="handleFileChange"
      />

      <!-- Editor -->
      <MessageEditor
        ref="editorRef"
        :placeholder="t.input.placeholder"
        :disabled="isLoading"
        :participants="participants"
        :current-user-id="currentUserId"
        :has-attachments="fileUpload.pendingAttachments.value.length > 0"
        @submit="handleSubmit"
        @update:is-empty="editorIsEmpty = $event"
        @attach="handleFileSelect"
        @arrow-up="handleArrowUp"
      />
    </template>
  </div>
</template>

<style scoped>
.chat-input {
  border-top: 1px solid var(--mtchat-border);
  background: var(--mtchat-bg);
  padding: 12px;
}

/* Join prompt */
.chat-input__join-prompt {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  padding: 16px;
}

.chat-input__join-prompt p {
  margin: 0;
  color: var(--mtchat-text-secondary);
  font-size: 14px;
}

.chat-input__btn {
  padding: 8px 16px;
  border-radius: 8px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  border: none;
}

.chat-input__btn--primary {
  background: var(--mtchat-primary);
  color: white;
}

.chat-input__btn--primary:hover {
  opacity: 0.9;
}

/* Preview (reply/edit) */
.chat-input__preview {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  padding: 8px 12px;
  background: var(--mtchat-bg-secondary);
  border-bottom: 1px solid var(--mtchat-border);
}

.chat-input__preview-indicator {
  width: 3px;
  min-height: 32px;
  border-radius: 2px;
  flex-shrink: 0;
}

.chat-input__preview--reply .chat-input__preview-indicator {
  background: var(--mtchat-primary);
}

.chat-input__preview--edit .chat-input__preview-indicator {
  background: #f59e0b;
}

.chat-input__preview-content {
  flex: 1;
  min-width: 0;
}

.chat-input__preview-label {
  font-size: 12px;
  font-weight: 600;
  color: #f59e0b;
  margin-bottom: 2px;
}

.chat-input__preview-author {
  font-size: 12px;
  font-weight: 600;
  color: var(--mtchat-primary);
  margin-bottom: 2px;
}

.chat-input__preview-text {
  font-size: 13px;
  color: var(--mtchat-text-secondary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.chat-input__preview-cancel {
  padding: 4px;
  background: none;
  border: none;
  color: var(--mtchat-text-secondary);
  cursor: pointer;
  border-radius: 4px;
  flex-shrink: 0;
}

.chat-input__preview-cancel:hover {
  background: var(--mtchat-bg-hover);
  color: var(--mtchat-text);
}

/* File input (hidden) */
.chat-input__file-input {
  display: none;
}
</style>
