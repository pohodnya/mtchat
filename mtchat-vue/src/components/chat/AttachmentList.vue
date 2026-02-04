<template>
  <div v-if="attachments.length > 0" class="attachment-list">
    <!-- Image grid -->
    <div v-if="imageAttachments.length > 0" class="images-grid">
      <AttachmentThumbnail
        v-for="(att, index) in imageAttachments"
        :key="att.id"
        :attachment="att"
        @click="$emit('open-gallery', index)"
      />
    </div>

    <!-- Files list -->
    <div v-if="fileAttachments.length > 0" class="files-list">
      <AttachmentFile
        v-for="att in fileAttachments"
        :key="att.id"
        :attachment="att"
        @click="$emit('open-file', att)"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { Attachment } from '../../types'
import { getAttachmentType } from '../../types'
import AttachmentThumbnail from './AttachmentThumbnail.vue'
import AttachmentFile from './AttachmentFile.vue'

const props = defineProps<{
  attachments: Attachment[]
}>()

defineEmits<{
  (e: 'open-gallery', index: number): void
  (e: 'open-file', attachment: Attachment): void
}>()

const imageAttachments = computed(() =>
  props.attachments.filter((a) => getAttachmentType(a.content_type) === 'image')
)

const fileAttachments = computed(() =>
  props.attachments.filter((a) => getAttachmentType(a.content_type) !== 'image')
)
</script>

<style scoped>
.attachment-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-top: 8px;
}

.images-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.files-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
</style>
