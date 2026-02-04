<template>
  <div class="attachment-thumbnail" @click="$emit('click')">
    <img
      :src="attachment.thumbnail_url || attachment.url"
      :alt="attachment.filename"
      loading="lazy"
      @error="handleImageError"
    />
    <div class="thumbnail-overlay">
      <i class="pi pi-search-plus" />
    </div>
    <!-- Size indicator for large images -->
    <div v-if="attachment.width && attachment.height" class="size-badge">
      {{ attachment.width }}x{{ attachment.height }}
    </div>
  </div>
</template>

<script setup lang="ts">
import type { Attachment } from '../../types'

defineProps<{
  attachment: Attachment
}>()

defineEmits<{
  (e: 'click'): void
}>()

function handleImageError(event: Event) {
  const img = event.target as HTMLImageElement
  img.style.display = 'none'
}
</script>

<style scoped>
.attachment-thumbnail {
  position: relative;
  width: 120px;
  height: 120px;
  border-radius: 8px;
  overflow: hidden;
  cursor: pointer;
  background: rgba(255, 255, 255, 0.05);
}

.attachment-thumbnail img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  transition: transform 0.2s;
}

.attachment-thumbnail:hover img {
  transform: scale(1.05);
}

.thumbnail-overlay {
  position: absolute;
  inset: 0;
  background: rgba(0, 0, 0, 0.4);
  display: flex;
  align-items: center;
  justify-content: center;
  opacity: 0;
  transition: opacity 0.2s;
}

.attachment-thumbnail:hover .thumbnail-overlay {
  opacity: 1;
}

.thumbnail-overlay i {
  color: white;
  font-size: 24px;
}

.size-badge {
  position: absolute;
  bottom: 4px;
  right: 4px;
  padding: 2px 6px;
  background: rgba(0, 0, 0, 0.6);
  color: white;
  font-size: 10px;
  border-radius: 4px;
}
</style>
