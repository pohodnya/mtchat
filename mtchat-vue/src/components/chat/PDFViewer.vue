<template>
  <Teleport to="body">
    <div v-if="show" class="pdf-overlay" @click="handleOverlayClick">
      <div class="pdf-container">
        <!-- Close button -->
        <button class="pdf-close" @click="$emit('close')">
          <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M18 6L6 18M6 6l12 12" />
          </svg>
        </button>

        <!-- PDF content -->
        <div ref="scrollContainerRef" class="pdf-content" @click.stop>
          <div v-if="loading" class="pdf-loading">
            <div class="loading-spinner" />
            <span>Loading PDF...</span>
          </div>
          <div v-else-if="error" class="pdf-error">
            <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="12" cy="12" r="10" />
              <path d="M12 8v4M12 16h.01" />
            </svg>
            <span>{{ error }}</span>
          </div>
          <div
            v-else
            class="pdf-pages-wrapper"
            :style="{ height: `${pagesHeight * zoomLevel}px` }"
          >
            <div
              class="pdf-pages"
              :style="{ transform: `scale(${zoomLevel})`, transformOrigin: 'top center' }"
            >
              <canvas
                v-for="page in totalPages"
                :key="page"
                ref="canvasElements"
                class="pdf-page"
              />
            </div>
          </div>
        </div>

        <!-- Footer -->
        <div class="pdf-footer" @click.stop>
          <span class="pdf-filename" :title="filename">{{ filename }}</span>
          <span class="pdf-pages-count">{{ totalPages }} {{ totalPages === 1 ? 'page' : 'pages' }}</span>

          <!-- Right side controls -->
          <div class="pdf-controls">
            <!-- Zoom controls -->
            <div class="pdf-zoom">
              <button
                class="pdf-zoom-btn"
                :disabled="zoomLevel <= 0.5"
                title="Zoom out (−)"
                @click="zoomOut"
              >
                <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <circle cx="11" cy="11" r="8" />
                  <path d="M21 21l-4.35-4.35M8 11h6" />
                </svg>
              </button>
              <span class="pdf-zoom-level">{{ Math.round(zoomLevel * 100) }}%</span>
              <button
                class="pdf-zoom-btn"
                :disabled="zoomLevel >= 3"
                title="Zoom in (+)"
                @click="zoomIn"
              >
                <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <circle cx="11" cy="11" r="8" />
                  <path d="M21 21l-4.35-4.35M11 8v6M8 11h6" />
                </svg>
              </button>
            </div>

            <!-- Download button -->
            <a
              :href="url"
              download
              class="pdf-download"
              title="Download"
            >
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M7 10l5 5 5-5M12 15V3" />
              </svg>
            </a>
          </div>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, shallowRef, watch, onMounted, onUnmounted, nextTick } from 'vue'
import * as pdfjsLib from 'pdfjs-dist'
// @ts-ignore - Vite handles this import
import pdfjsWorker from 'pdfjs-dist/build/pdf.worker.min.mjs?url'

// Set worker source (use local file, not CDN)
pdfjsLib.GlobalWorkerOptions.workerSrc = pdfjsWorker

const props = defineProps<{
  show: boolean
  url: string
  filename: string
}>()

const emit = defineEmits<{
  (e: 'close'): void
}>()

// Refs
const scrollContainerRef = ref<HTMLDivElement | null>(null)
const canvasElements = ref<HTMLCanvasElement[]>([])

// State
const loading = ref(false)
const error = ref<string | null>(null)
const pdfDoc = shallowRef<pdfjsLib.PDFDocumentProxy | null>(null)
const totalPages = ref(0)
const zoomLevel = ref(1)
const baseScale = ref(1) // Scale to fit width
const pagesHeight = ref(0) // Total height of all pages for wrapper sizing

// Load PDF when shown
watch(
  () => props.show,
  async (show) => {
    if (show && props.url) {
      await loadPdf()
    } else {
      // Reset state when closed
      pdfDoc.value = null
      totalPages.value = 0
      error.value = null
      zoomLevel.value = 1
      pagesHeight.value = 0
    }
  }
)

async function loadPdf() {
  loading.value = true
  error.value = null

  try {
    const loadingTask = pdfjsLib.getDocument(props.url)
    pdfDoc.value = await loadingTask.promise
    totalPages.value = pdfDoc.value.numPages

    // Turn off loading so Vue renders the canvas elements (v-else block)
    loading.value = false

    // Wait for Vue to create canvas elements
    await nextTick()

    await calculateFitScale()
    await renderAllPages()
  } catch (e) {
    console.error('Failed to load PDF:', e)
    error.value = 'Failed to load PDF'
    loading.value = false
  }
}

async function calculateFitScale() {
  if (!pdfDoc.value || !scrollContainerRef.value) return

  // Get first page to determine dimensions
  const page = await pdfDoc.value.getPage(1)
  const viewport = page.getViewport({ scale: 1 })

  // Calculate scale to fit container width (with padding)
  const containerWidth = scrollContainerRef.value.clientWidth - 80 // 40px padding each side
  baseScale.value = containerWidth / viewport.width

  // Ensure minimum scale
  baseScale.value = Math.max(0.5, Math.min(2, baseScale.value))
  zoomLevel.value = 1 // Reset zoom to 100% of fit-width
}

async function renderAllPages() {
  if (!pdfDoc.value) return

  // Wait for canvases to be mounted
  await nextTick()

  for (let pageNum = 1; pageNum <= totalPages.value; pageNum++) {
    await renderPage(pageNum)
  }

  // Calculate total height for wrapper (sum of all canvas heights + gaps + padding)
  const gap = 16 // gap between pages in css
  const padding = 40 // 20px top + 20px bottom padding in pdf-pages
  let totalHeight = padding
  for (const canvas of canvasElements.value) {
    totalHeight += canvas.height
  }
  if (totalPages.value > 1) {
    totalHeight += gap * (totalPages.value - 1) // gaps between pages
  }
  pagesHeight.value = totalHeight
}

async function renderPage(pageNum: number) {
  if (!pdfDoc.value) return

  // Array is 0-indexed, pages are 1-indexed
  const canvas = canvasElements.value[pageNum - 1]
  if (!canvas) {
    console.warn(`Canvas not found for page ${pageNum}`)
    return
  }

  try {
    const page = await pdfDoc.value.getPage(pageNum)
    // Render at base scale (fit-width scale), zoom is applied via CSS transform
    const viewport = page.getViewport({ scale: baseScale.value })

    const context = canvas.getContext('2d')!

    // Set canvas dimensions
    canvas.height = viewport.height
    canvas.width = viewport.width

    // Render page
    // @ts-ignore - pdfjs-dist types are inconsistent across versions
    await page.render({
      canvasContext: context,
      viewport: viewport,
    }).promise
  } catch (e) {
    console.error(`Failed to render page ${pageNum}:`, e)
  }
}

function zoomIn() {
  if (zoomLevel.value < 3) {
    zoomLevel.value = Math.min(3, zoomLevel.value + 0.25)
  }
}

function zoomOut() {
  if (zoomLevel.value > 0.5) {
    zoomLevel.value = Math.max(0.5, zoomLevel.value - 0.25)
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
    case '+':
    case '=':
      zoomIn()
      break
    case '-':
      zoomOut()
      break
  }
}

function handleWheel(e: WheelEvent) {
  if (!props.show) return

  // Ctrl+wheel for zoom
  if (e.ctrlKey || e.metaKey) {
    e.preventDefault()
    if (e.deltaY < 0) {
      zoomIn()
    } else {
      zoomOut()
    }
  }
}

onMounted(() => {
  document.addEventListener('keydown', handleKeydown)
  document.addEventListener('wheel', handleWheel, { passive: false })
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeydown)
  document.removeEventListener('wheel', handleWheel)
})
</script>

<style scoped>
.pdf-overlay {
  position: fixed;
  inset: 0;
  z-index: 10000;
  background: rgba(0, 0, 0, 0.9);
  display: flex;
  align-items: center;
  justify-content: center;
}

.pdf-container {
  position: relative;
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
}

.pdf-close {
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

.pdf-close:hover {
  background: rgba(255, 255, 255, 0.2);
}

.pdf-content {
  flex: 1;
  overflow: auto;
  padding: 60px 40px 80px;
}

.pdf-pages-wrapper {
  display: flex;
  justify-content: center;
  overflow: hidden;
}

.pdf-pages {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
  padding: 20px 0;
  transition: transform 0.1s ease-out;
}

.pdf-page {
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.4);
  background: white;
}

.pdf-loading,
.pdf-error {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 16px;
  color: rgba(255, 255, 255, 0.6);
  height: 100%;
}

.loading-spinner {
  width: 40px;
  height: 40px;
  border: 3px solid rgba(255, 255, 255, 0.1);
  border-top-color: #4fc3f7;
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.pdf-error svg {
  color: #ef5350;
}

.pdf-footer {
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

.pdf-filename {
  flex: 1;
  font-size: 14px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.pdf-pages-count {
  font-size: 14px;
  color: rgba(255, 255, 255, 0.7);
}

.pdf-controls {
  display: flex;
  align-items: center;
  gap: 12px;
}

.pdf-zoom {
  display: flex;
  align-items: center;
  gap: 4px;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 18px;
  padding: 4px;
}

.pdf-zoom-btn {
  width: 28px;
  height: 28px;
  border: none;
  background: transparent;
  color: rgba(255, 255, 255, 0.8);
  border-radius: 50%;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s;
}

.pdf-zoom-btn:hover:not(:disabled) {
  background: rgba(255, 255, 255, 0.15);
  color: white;
}

.pdf-zoom-btn:disabled {
  opacity: 0.3;
  cursor: not-allowed;
}

.pdf-zoom-level {
  font-size: 12px;
  color: rgba(255, 255, 255, 0.8);
  min-width: 40px;
  text-align: center;
}

.pdf-download {
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

.pdf-download:hover {
  background: rgba(255, 255, 255, 0.2);
}
</style>
