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
        <i :class="getFileIcon(attachment.contentType)" />
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
      <i v-if="attachment.status === 'uploaded'" class="pi pi-check status-icon success" />
      <i
        v-if="attachment.status === 'error'"
        class="pi pi-exclamation-triangle status-icon error"
        :title="attachment.error"
        @click="$emit('retry', attachment.id)"
      />

      <!-- Remove button -->
      <button
        class="remove-btn"
        :disabled="attachment.status === 'uploading'"
        @click="$emit('remove', attachment.id)"
      >
        <i class="pi pi-times" />
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { PendingAttachment } from '../../types'

defineProps<{
  attachments: PendingAttachment[]
}>()

defineEmits<{
  (e: 'remove', id: string): void
  (e: 'retry', id: string): void
}>()

function getFileIcon(contentType: string): string {
  if (contentType === 'application/pdf') return 'pi pi-file-pdf'
  if (contentType.startsWith('image/')) return 'pi pi-image'
  return 'pi pi-file'
}

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
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 8px;
  transition: all 0.2s;
}

.preview-item.uploading {
  opacity: 0.7;
}

.preview-item.uploaded {
  border-color: rgba(102, 187, 106, 0.3);
}

.preview-item.error {
  border-color: rgba(239, 83, 80, 0.3);
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
  background: rgba(255, 255, 255, 0.05);
  border-radius: 4px;
}

.preview-file i {
  font-size: 24px;
  color: #888;
}

.preview-name {
  margin-top: 4px;
  font-size: 10px;
  color: #888;
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
  background: rgba(255, 255, 255, 0.1);
  border-radius: 0 0 8px 8px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: #4fc3f7;
  transition: width 0.2s;
}

.status-icon {
  position: absolute;
  top: 4px;
  right: 20px;
  font-size: 12px;
}

.status-icon.success {
  color: #66bb6a;
}

.status-icon.error {
  color: #ef5350;
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

.remove-btn i {
  font-size: 10px;
}
</style>
