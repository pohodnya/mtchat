<template>
  <div class="attachment-file" @click="$emit('click')">
    <div class="file-icon" :class="iconClass">
      <i :class="icon" />
    </div>
    <div class="file-info">
      <span class="file-name" :title="attachment.filename">{{ attachment.filename }}</span>
      <span class="file-size">{{ formatFileSize(attachment.size) }}</span>
    </div>
    <button class="download-btn" title="Download" @click.stop="download">
      <i class="pi pi-download" />
    </button>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { Attachment } from '../../types'
import { formatFileSize } from '../../types'

const props = defineProps<{
  attachment: Attachment
}>()

defineEmits<{
  (e: 'click'): void
}>()

const icon = computed(() => {
  if (props.attachment.content_type === 'application/pdf') {
    return 'pi pi-file-pdf'
  }
  return 'pi pi-file'
})

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
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s;
  max-width: 280px;
}

.attachment-file:hover {
  background: rgba(255, 255, 255, 0.08);
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
  background: rgba(239, 68, 68, 0.2);
  color: #ef4444;
}

.file-icon.generic {
  background: rgba(79, 195, 247, 0.2);
  color: #4fc3f7;
}

.file-icon i {
  font-size: 20px;
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
  color: #e0e0e0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.file-size {
  font-size: 12px;
  color: #888;
}

.download-btn {
  width: 32px;
  height: 32px;
  border-radius: 6px;
  border: none;
  background: transparent;
  color: #888;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  transition: all 0.2s;
}

.download-btn:hover {
  background: rgba(255, 255, 255, 0.1);
  color: #fff;
}
</style>
