<template>
  <div v-if="attachments.length > 0" class="attachment-preview">
    <div
      v-for="attachment in attachments"
      :key="attachment.id"
      class="preview-item"
      :class="{
        uploading: attachment.status === 'uploading',
        uploaded: attachment.status === 'uploaded',
        error: attachment.status === 'error',
      }"
    >
      <!-- Image preview -->
      <img
        v-if="attachment.previewUrl"
        :src="attachment.previewUrl"
        :alt="attachment.filename"
        class="preview-image"
      />

      <!-- File icon -->
      <div v-else class="preview-file">
        <Icon :name="getFileIconName(attachment.contentType)" :size="24" />
      </div>

      <!-- Filename -->
      <span class="preview-name" :title="attachment.filename">
        {{ truncateFilename(attachment.filename) }}
      </span>

      <!-- Progress bar -->
      <div v-if="attachment.status === 'uploading'" class="progress-bar">
        <div class="progress-fill" :style="{ width: `${attachment.progress}%` }" />
      </div>

      <!-- Status icons -->
      <Icon v-if="attachment.status === 'uploaded'" name="check" :size="12" class="status-icon success" />
      <Icon
        v-if="attachment.status === 'error'"
        name="error"
        :size="12"
        class="status-icon error"
        :title="attachment.error"
        @click="$emit('retry', attachment.id)"
      />

      <!-- Remove button -->
      <button
        class="remove-btn"
        :disabled="attachment.status === 'uploading'"
        @click="$emit('remove', attachment.id)"
      >
        <Icon name="close" :size="10" />
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { PendingAttachment } from '../../types'
import { getFileIconName } from '../../types'
import Icon from '../Icon.vue'

defineProps<{
  attachments: PendingAttachment[]
}>()

defineEmits<{
  (e: 'remove', id: string): void
  (e: 'retry', id: string): void
}>()

function truncateFilename(name: string, maxLength = 20): string {
  if (name.length <= maxLength) return name
  const ext = name.split('.').pop() || ''
  const base = name.slice(0, name.length - ext.length - 1)
  const truncated = base.slice(0, maxLength - ext.length - 4) + '...'
  return `${truncated}.${ext}`
}
</script>

<style scoped>
.attachment-preview {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  padding: 8px 0;
}

.preview-item {
  position: relative;
  display: flex;
  flex-direction: column;
  align-items: center;
  width: 80px;
  padding: 8px;
  background: var(--mtchat-bg-secondary);
  border: 1px solid var(--mtchat-border);
  border-radius: 8px;
  transition: all 0.2s;
}

.preview-item.uploading {
  opacity: 0.7;
}

.preview-item.uploaded {
  border-color: rgba(102, 187, 106, 0.5);
}

.preview-item.error {
  border-color: rgba(239, 83, 80, 0.5);
  background: rgba(239, 83, 80, 0.1);
}

.preview-image {
  width: 60px;
  height: 60px;
  object-fit: cover;
  border-radius: 4px;
}

.preview-file {
  width: 60px;
  height: 60px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--mtchat-bg-hover);
  border-radius: 4px;
  color: var(--mtchat-text-secondary);
}

.preview-name {
  margin-top: 4px;
  font-size: 10px;
  color: var(--mtchat-text-secondary);
  text-align: center;
  word-break: break-all;
  line-height: 1.2;
}

.progress-bar {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  height: 3px;
  background: var(--mtchat-border);
  border-radius: 0 0 8px 8px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: var(--mtchat-primary);
  transition: width 0.2s;
}

.status-icon {
  position: absolute;
  top: 4px;
  right: 20px;
}

.status-icon.success {
  color: var(--mtchat-success);
}

.status-icon.error {
  color: var(--mtchat-danger);
  cursor: pointer;
}

.remove-btn {
  position: absolute;
  top: 4px;
  right: 4px;
  width: 20px;
  height: 20px;
  padding: 0;
  border: none;
  background: rgba(0, 0, 0, 0.7);
  color: #fff;
  border-radius: 50%;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  opacity: 1;
  transition: background 0.2s;
}

.remove-btn:hover {
  background: rgba(239, 68, 68, 0.9);
}

.remove-btn:disabled {
  cursor: not-allowed;
  opacity: 0.5;
}
</style>
