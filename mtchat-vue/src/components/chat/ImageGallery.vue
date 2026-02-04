<template>
  <Teleport to="body">
    <div v-if="show" class="gallery-overlay" @click="handleOverlayClick">
      <div class="gallery-container">
        <!-- Close button -->
        <button class="gallery-close" @click="$emit('close')">
          <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M18 6L6 18M6 6l12 12" />
          </svg>
        </button>

        <!-- Navigation -->
        <button
          v-if="images.length > 1"
          class="gallery-nav gallery-nav--prev"
          :disabled="currentIndex === 0"
          @click.stop="prev"
        >
          <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M15 18l-6-6 6-6" />
          </svg>
        </button>

        <button
          v-if="images.length > 1"
          class="gallery-nav gallery-nav--next"
          :disabled="currentIndex === images.length - 1"
          @click.stop="next"
        >
          <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M9 18l6-6-6-6" />
          </svg>
        </button>

        <!-- Image -->
        <div class="gallery-image-wrapper">
          <img
            v-if="currentImage"
            :src="currentImage.url"
            :alt="currentImage.filename"
            class="gallery-image"
            @click.stop
          />
        </div>

        <!-- Footer -->
        <div class="gallery-footer" @click.stop>
          <span class="gallery-filename">{{ currentImage?.filename }}</span>
          <span v-if="images.length > 1" class="gallery-counter">
            {{ currentIndex + 1 }} / {{ images.length }}
          </span>
          <a
            v-if="currentImage"
            :href="currentImage.url"
            download
            class="gallery-download"
            target="_blank"
          >
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M7 10l5 5 5-5M12 15V3" />
            </svg>
          </a>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { computed, watch, onMounted, onUnmounted } from 'vue'
import type { Attachment } from '../../types'

const props = defineProps<{
  show: boolean
  images: Attachment[]
  initialIndex?: number
}>()

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'update:index', index: number): void
}>()

const currentIndex = defineModel<number>('index', { default: 0 })

const currentImage = computed(() => props.images[currentIndex.value])

function prev() {
  if (currentIndex.value > 0) {
    currentIndex.value--
  }
}

function next() {
  if (currentIndex.value < props.images.length - 1) {
    currentIndex.value++
  }
}

function handleOverlayClick(e: MouseEvent) {
  if (e.target === e.currentTarget) {
    emit('close')
  }
}

function handleKeydown(e: KeyboardEvent) {
  if (!props.show) return

  switch (e.key) {
    case 'Escape':
      emit('close')
      break
    case 'ArrowLeft':
      prev()
      break
    case 'ArrowRight':
      next()
      break
  }
}

// Set initial index when opened
watch(
  () => props.show,
  (show) => {
    if (show && props.initialIndex !== undefined) {
      currentIndex.value = props.initialIndex
    }
  }
)

onMounted(() => {
  document.addEventListener('keydown', handleKeydown)
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeydown)
})
</script>

<style scoped>
.gallery-overlay {
  position: fixed;
  inset: 0;
  z-index: 10000;
  background: rgba(0, 0, 0, 0.9);
  display: flex;
  align-items: center;
  justify-content: center;
}

.gallery-container {
  position: relative;
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
}

.gallery-close {
  position: absolute;
  top: 16px;
  right: 16px;
  z-index: 10;
  width: 40px;
  height: 40px;
  border: none;
  background: rgba(255, 255, 255, 0.1);
  color: white;
  border-radius: 50%;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background 0.2s;
}

.gallery-close:hover {
  background: rgba(255, 255, 255, 0.2);
}

.gallery-nav {
  position: absolute;
  top: 50%;
  transform: translateY(-50%);
  z-index: 10;
  width: 48px;
  height: 48px;
  border: none;
  background: rgba(255, 255, 255, 0.1);
  color: white;
  border-radius: 50%;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background 0.2s;
}

.gallery-nav:disabled {
  opacity: 0.3;
  cursor: not-allowed;
}

.gallery-nav:not(:disabled):hover {
  background: rgba(255, 255, 255, 0.2);
}

.gallery-nav--prev {
  left: 16px;
}

.gallery-nav--next {
  right: 16px;
}

.gallery-image-wrapper {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 60px 80px;
  overflow: hidden;
}

.gallery-image {
  max-width: 100%;
  max-height: 100%;
  object-fit: contain;
}

.gallery-footer {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  padding: 16px 24px;
  background: linear-gradient(transparent, rgba(0, 0, 0, 0.7));
  display: flex;
  align-items: center;
  gap: 16px;
  color: white;
}

.gallery-filename {
  flex: 1;
  font-size: 14px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.gallery-counter {
  font-size: 14px;
  color: rgba(255, 255, 255, 0.7);
}

.gallery-download {
  width: 36px;
  height: 36px;
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.1);
  color: white;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background 0.2s;
  text-decoration: none;
}

.gallery-download:hover {
  background: rgba(255, 255, 255, 0.2);
}
</style>
