<template>
  <div class="attachment-file" @click="$emit('click')">
    <div class="file-icon" :class="iconClass">
      <Icon :name="icon" :size="20" />
    </div>
    <div class="file-info">
      <span class="file-name" :title="attachment.filename">{{ attachment.filename }}</span>
      <span class="file-size">{{ formatFileSize(attachment.size) }}</span>
    </div>
    <button class="download-btn" title="Download" @click.stop="download">
      <Icon name="download" :size="16" />
    </button>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { Attachment } from '../../types'
import { formatFileSize, getFileIconName } from '../../types'
import Icon from '../Icon.vue'

const props = defineProps<{
  attachment: Attachment
}>()

defineEmits<{
  (e: 'click'): void
}>()

const icon = computed(() => getFileIconName(props.attachment.content_type))

const iconClass = computed(() => {
  if (props.attachment.content_type === 'application/pdf') {
    return 'pdf'
  }
  return 'generic'
})

function download() {
  window.open(props.attachment.url, '_blank')
}
</script>

<style scoped>
.attachment-file {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px;
  background: var(--mtchat-bg-secondary);
  border: 1px solid var(--mtchat-border);
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s;
  max-width: 280px;
}

.attachment-file:hover {
  background: var(--mtchat-bg-hover);
}

.file-icon {
  width: 40px;
  height: 40px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.file-icon.pdf {
  background: color-mix(in srgb, var(--mtchat-danger) 15%, transparent);
  color: var(--mtchat-danger);
}

.file-icon.generic {
  background: rgba(59, 130, 246, 0.15);
  color: var(--mtchat-primary);
}

.file-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.file-name {
  font-size: 14px;
  font-weight: 500;
  color: var(--mtchat-text);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.file-size {
  font-size: 12px;
  color: var(--mtchat-text-secondary);
}

.download-btn {
  width: 32px;
  height: 32px;
  border-radius: 6px;
  border: none;
  background: transparent;
  color: var(--mtchat-text-secondary);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  transition: all 0.2s;
}

.download-btn:hover {
  background: var(--mtchat-bg-hover);
  color: var(--mtchat-primary);
}
</style>
