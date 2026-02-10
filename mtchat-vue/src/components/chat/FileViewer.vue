<template>
  <Teleport to="body">
    <div v-if="show" class="viewer-overlay" @click="handleOverlayClick">
      <div class="viewer-container">
        <!-- Close button -->
        <button class="viewer-close" :title="t.fileViewer.close" @click="$emit('close')">
          <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M18 6L6 18M6 6l12 12" />
          </svg>
        </button>

        <!-- Navigation -->
        <button
          v-if="files.length > 1"
          class="viewer-nav viewer-nav--prev"
          :disabled="currentIndex <= 0"
          :title="t.fileViewer.previous"
          @click.stop="prev"
        >
          <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M15 18l-6-6 6-6" />
          </svg>
        </button>

        <button
          v-if="files.length > 1"
          class="viewer-nav viewer-nav--next"
          :disabled="currentIndex >= files.length - 1"
          :title="t.fileViewer.next"
          @click.stop="next"
        >
          <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M9 18l6-6-6-6" />
          </svg>
        </button>

        <!-- Content area -->
        <div
          ref="contentRef"
          class="viewer-content"
          :class="{
            'viewer-content--draggable': canDrag,
            'viewer-content--ready': contentReady
          }"
          @mousedown="startDrag"
          @mousemove="onDrag"
          @mouseup="endDrag"
          @mouseleave="endDrag"
          @wheel="handleWheel"
          @click.stop
        >
          <!-- Image -->
          <template v-if="currentFileType === 'image'">
            <div
              class="viewer-image-wrapper"
              :style="contentTransformStyle"
            >
              <img
                v-if="currentFile"
                ref="imageRef"
                :src="currentFile.url"
                :alt="currentFile.filename"
                class="viewer-image"
                draggable="false"
                @load="onImageLoad"
              />
            </div>
          </template>

          <!-- PDF -->
          <template v-else-if="currentFileType === 'pdf'">
            <div v-if="pdfError" class="viewer-file-preview">
              <svg width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                <circle cx="12" cy="12" r="10" />
                <path d="M12 8v4M12 16h.01" />
              </svg>
              <span class="viewer-file-name">{{ pdfError }}</span>
            </div>
            <div
              v-else
              class="viewer-pdf-wrapper"
              :style="contentTransformStyle"
            >
              <div class="pdf-pages">
                <canvas
                  v-for="page in pdfTotalPages"
                  :key="`${currentFile?.id}-page-${page}`"
                  ref="pdfCanvasElements"
                  class="pdf-page"
                />
              </div>
            </div>
          </template>

          <!-- Other file types -->
          <template v-else>
            <div class="viewer-file-preview">
              <component :is="getFileIcon(currentFile?.content_type)" class="viewer-file-icon" />
              <a
                class="viewer-file-name viewer-file-name--clickable"
                href="#"
                @click.prevent="downloadCurrentFile"
              >
                {{ currentFile?.filename }}
              </a>
              <span class="viewer-file-type">{{ getFileTypeLabel(currentFile?.content_type) }}</span>
            </div>
          </template>
        </div>

        <!-- Footer -->
        <div class="viewer-footer" @click.stop>
          <span class="viewer-filename" :title="currentFile?.filename">{{ currentFile?.filename }}</span>

          <span v-if="files.length > 1" class="viewer-counter">
            {{ currentIndex + 1 }} / {{ files.length }}
          </span>

          <!-- PDF page count -->
          <span v-if="currentFileType === 'pdf' && !pdfLoading && !pdfError" class="viewer-pages-count">
            {{ pdfTotalPages }} {{ t.fileViewer.page }}
          </span>

          <!-- Zoom controls (for images and PDFs) -->
          <div v-if="currentFileType === 'image' || (currentFileType === 'pdf' && !pdfLoading && !pdfError)" class="viewer-zoom">
            <button
              class="viewer-zoom-btn"
              :disabled="zoom <= 0.5"
              :title="t.fileViewer.zoomOut"
              @click="zoomOut()"
            >
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <circle cx="11" cy="11" r="8" />
                <path d="M21 21l-4.35-4.35M8 11h6" />
              </svg>
            </button>
            <span class="viewer-zoom-level">{{ Math.round(zoom * 100) }}%</span>
            <button
              class="viewer-zoom-btn"
              :disabled="zoom >= 5"
              :title="t.fileViewer.zoomIn"
              @click="zoomIn()"
            >
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <circle cx="11" cy="11" r="8" />
                <path d="M21 21l-4.35-4.35M11 8v6M8 11h6" />
              </svg>
            </button>
            <button
              class="viewer-zoom-btn"
              :title="t.fileViewer.resetZoom"
              @click="resetZoom"
            >
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8" />
                <path d="M3 3v5h5" />
              </svg>
            </button>
          </div>

          <!-- Download button -->
          <button
            v-if="currentFile"
            class="viewer-download"
            :title="t.fileViewer.download"
            @click="downloadCurrentFile"
          >
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M7 10l5 5 5-5M12 15V3" />
            </svg>
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, shallowRef, computed, watch, onMounted, onUnmounted, nextTick, h, type FunctionalComponent } from 'vue'
import type { Attachment } from '../../types'
import { getAttachmentType } from '../../types'
import { useI18n } from '../../i18n'
import * as pdfjsLib from 'pdfjs-dist'
// @ts-ignore - Vite handles this import
import pdfjsWorker from 'pdfjs-dist/build/pdf.worker.min.mjs?url'

// Set worker source
pdfjsLib.GlobalWorkerOptions.workerSrc = pdfjsWorker

const { t } = useI18n()

const props = defineProps<{
  show: boolean
  files: Attachment[]
  initialIndex?: number
}>()

const emit = defineEmits<{
  (e: 'close'): void
}>()

// Refs
const contentRef = ref<HTMLDivElement | null>(null)
const imageRef = ref<HTMLImageElement | null>(null)
const pdfCanvasElements = ref<HTMLCanvasElement[]>([])

// Navigation state
const currentIndex = ref(0)
const contentReady = ref(false) // For fade-in transition

const currentFile = computed(() => props.files[currentIndex.value])

const currentFileType = computed(() => {
  if (!currentFile.value) return 'unknown'
  const type = getAttachmentType(currentFile.value.content_type)
  if (type === 'image') return 'image'
  if (currentFile.value.content_type === 'application/pdf') return 'pdf'
  return 'other'
})

// Zoom and pan state
const zoom = ref(1)
const panX = ref(0)
const panY = ref(0)
const isDragging = ref(false)
const dragStartX = ref(0)
const dragStartY = ref(0)
const dragStartPanX = ref(0)
const dragStartPanY = ref(0)

// Allow drag for zoomed content OR for PDFs (which may be longer than viewport)
const canDrag = computed(() => zoom.value > 1 || currentFileType.value === 'pdf')

// Track if we're in interactive zoom/pan (disable CSS transition)
const isInteracting = ref(false)
let interactionTimeout: ReturnType<typeof setTimeout> | null = null

function setInteracting(value: boolean) {
  if (value) {
    isInteracting.value = true
    if (interactionTimeout) clearTimeout(interactionTimeout)
    interactionTimeout = setTimeout(() => {
      isInteracting.value = false
    }, 150)
  }
}

const contentTransformStyle = computed(() => ({
  transform: `scale(${zoom.value}) translate(${panX.value}px, ${panY.value}px)`,
  cursor: isDragging.value ? 'grabbing' : (canDrag.value ? 'grab' : 'default'),
  transition: (isInteracting.value || isDragging.value) ? 'none' : 'transform 0.15s ease-out',
}))

// PDF state
const pdfDoc = shallowRef<pdfjsLib.PDFDocumentProxy | null>(null)
const pdfLoading = ref(false)
const pdfError = ref<string | null>(null)
const pdfTotalPages = ref(0)
const pdfRenderTasks = ref<Map<number, any>>(new Map()) // Track active render tasks

// High quality render scale (device pixel ratio aware)
const PDF_RENDER_SCALE = Math.max(2, window.devicePixelRatio || 1)

// Watch show prop
watch(
  () => props.show,
  (show) => {
    if (show) {
      currentIndex.value = props.initialIndex ?? 0
      loadCurrentFile()
    } else {
      resetState()
    }
  }
)

// Watch current file change
watch(currentIndex, () => {
  if (props.show) {
    loadCurrentFile()
  }
})

function resetState() {
  pdfDoc.value = null
  pdfTotalPages.value = 0
  pdfError.value = null
  resetZoom()
}

function resetZoom() {
  zoom.value = 1
  panX.value = 0
  panY.value = 0
}

function scrollToTop() {
  const { height: contentHeight } = cachedContentSize.value
  const { height: containerHeight } = cachedContainerSize.value

  if (contentHeight > containerHeight) {
    // Pan to show top of content (positive panY moves content down, showing more top)
    panY.value = (contentHeight - containerHeight) / 2 / zoom.value
  }
}

// Cancel all active PDF render tasks
function cancelAllRenderTasks() {
  for (const task of pdfRenderTasks.value.values()) {
    try {
      task.cancel()
    } catch (e) {
      // Ignore cancellation errors
    }
  }
  pdfRenderTasks.value.clear()
}

async function loadCurrentFile() {
  // Fade out before loading
  contentReady.value = false

  // Cancel any ongoing renders
  cancelAllRenderTasks()

  resetZoom()
  pdfDoc.value = null
  pdfTotalPages.value = 0
  pdfError.value = null
  pdfCanvasElements.value = [] // Clear canvas refs to prevent reuse issues

  if (currentFileType.value === 'pdf') {
    await loadPdf()
    // PDF: fade in after all pages rendered (handled in renderAllPdfPages)
  } else if (currentFileType.value === 'image') {
    // Image: fade in after image loads (handled in onImageLoad)
  } else {
    // Other file types: fade in immediately
    await nextTick()
    contentReady.value = true
  }
}

// Image handling
function onImageLoad() {
  // Image loaded, reset zoom and cache sizes
  resetZoom()
  nextTick(() => {
    updateCachedSizes()
    contentReady.value = true
  })
}

// PDF loading
async function loadPdf() {
  if (!currentFile.value) return

  pdfLoading.value = true
  pdfError.value = null

  try {
    const loadingTask = pdfjsLib.getDocument(currentFile.value.url)
    pdfDoc.value = await loadingTask.promise
    pdfTotalPages.value = pdfDoc.value.numPages

    // Hide wrapper content (opacity 0) while rendering
    pdfLoading.value = false

    await nextTick()
    await renderAllPdfPages()
  } catch (e) {
    console.error('Failed to load PDF:', e)
    pdfError.value = 'Failed to load PDF'
    pdfLoading.value = false
  }
}

async function renderAllPdfPages() {
  if (!pdfDoc.value) return

  await nextTick()

  for (let pageNum = 1; pageNum <= pdfTotalPages.value; pageNum++) {
    await renderPdfPage(pageNum)
  }

  // Cache sizes after all pages rendered
  await nextTick()
  updateCachedSizes()

  // Position PDF at the top of first page
  scrollToTop()

  // Delay to ensure rendering is fully complete
  // await new Promise(resolve => setTimeout(resolve, 1000))

  contentReady.value = true
}

async function renderPdfPage(pageNum: number) {
  if (!pdfDoc.value) return

  const canvas = pdfCanvasElements.value[pageNum - 1]
  if (!canvas) return

  // Cancel any existing render task for this page
  const existingTask = pdfRenderTasks.value.get(pageNum)
  if (existingTask) {
    try {
      existingTask.cancel()
    } catch (e) {
      // Ignore
    }
    pdfRenderTasks.value.delete(pageNum)
  }

  try {
    const page = await pdfDoc.value.getPage(pageNum)

    // Get viewport as-is (no rotation manipulation)
    const viewport = page.getViewport({ scale: PDF_RENDER_SCALE })

    // Set canvas dimensions first (this also clears the canvas)
    canvas.width = viewport.width
    canvas.height = viewport.height

    // Get context after setting dimensions
    const context = canvas.getContext('2d')!

    // Calculate display size: limit to 60vw max
    const maxDisplayWidth = window.innerWidth * 0.6
    const naturalWidth = viewport.width / PDF_RENDER_SCALE
    const naturalHeight = viewport.height / PDF_RENDER_SCALE

    let displayWidth = naturalWidth
    let displayHeight = naturalHeight

    if (naturalWidth > maxDisplayWidth) {
      const scale = maxDisplayWidth / naturalWidth
      displayWidth = maxDisplayWidth
      displayHeight = naturalHeight * scale
    }

    canvas.style.width = `${displayWidth}px`
    canvas.style.height = `${displayHeight}px`

    // Create render task
    // @ts-ignore - pdfjs-dist types are inconsistent across versions
    const renderTask = page.render({
      canvasContext: context,
      viewport: viewport,
    })

    // Track the task
    pdfRenderTasks.value.set(pageNum, renderTask)

    // Wait for completion
    await renderTask.promise

    // Remove from tracking after successful completion
    pdfRenderTasks.value.delete(pageNum)
  } catch (e: any) {
    // Ignore cancellation errors
    if (e?.name !== 'RenderingCancelledException') {
      console.error(`Failed to render page ${pageNum}:`, e)
    }
  }
}

// Zoom functions
const MIN_ZOOM = 0.5
const MAX_ZOOM = 5

function zoomAt(newZoom: number, clientX?: number, clientY?: number) {
  newZoom = Math.max(MIN_ZOOM, Math.min(MAX_ZOOM, newZoom))
  if (newZoom === zoom.value) return

  // Zoom to cursor position if provided
  if (clientX !== undefined && clientY !== undefined && contentRef.value) {
    const rect = contentRef.value.getBoundingClientRect()
    const centerX = rect.left + rect.width / 2
    const centerY = rect.top + rect.height / 2

    // Point relative to center, in "content" coordinates
    const pointX = (clientX - centerX) / zoom.value - panX.value
    const pointY = (clientY - centerY) / zoom.value - panY.value

    // After zoom, adjust pan so the same point stays under cursor
    panX.value = (clientX - centerX) / newZoom - pointX
    panY.value = (clientY - centerY) / newZoom - pointY
  }

  zoom.value = newZoom

  // Reset pan if zoomed out to 1x (except PDFs which may be tall)
  if (zoom.value <= 1 && currentFileType.value !== 'pdf') {
    panX.value = 0
    panY.value = 0
  }

  clampPan()
}

function zoomIn() {
  zoomAt(zoom.value * 1.25)
}

function zoomOut() {
  zoomAt(zoom.value / 1.25)
}

function handleZoomWheel(deltaY: number, clientX: number, clientY: number) {
  setInteracting(true)
  // Zoom factor: ~10% per 100px of scroll
  const factor = Math.pow(1.01, -deltaY)
  zoomAt(zoom.value * factor, clientX, clientY)
}

// Cached content dimensions (updated on content change, not on every pan)
const cachedContentSize = ref({ width: 0, height: 0 })
const cachedContainerSize = ref({ width: 0, height: 0 })

function updateCachedSizes() {
  if (!contentRef.value) return

  const containerRect = contentRef.value.getBoundingClientRect()
  cachedContainerSize.value = { width: containerRect.width, height: containerRect.height }

  let contentWidth = 0
  let contentHeight = 0

  if (currentFileType.value === 'image' && imageRef.value) {
    contentWidth = imageRef.value.offsetWidth
    contentHeight = imageRef.value.offsetHeight
  } else if (currentFileType.value === 'pdf' && pdfCanvasElements.value.length > 0) {
    const firstCanvas = pdfCanvasElements.value[0]
    contentWidth = firstCanvas?.offsetWidth || 0
    contentHeight = pdfCanvasElements.value.reduce((sum, canvas) => sum + (canvas?.offsetHeight || 0), 0)
    contentHeight += (pdfCanvasElements.value.length - 1) * 16
  }

  cachedContentSize.value = { width: contentWidth, height: contentHeight }
}

function clampPan() {
  const { width: contentWidth, height: contentHeight } = cachedContentSize.value
  const { width: containerWidth, height: containerHeight } = cachedContainerSize.value

  if (contentWidth === 0 || containerWidth === 0) return

  const scaledWidth = contentWidth * zoom.value
  const scaledHeight = contentHeight * zoom.value

  const excessX = Math.max(0, (scaledWidth - containerWidth) / 2)
  const excessY = Math.max(0, (scaledHeight - containerHeight) / 2)

  const maxPanX = excessX / zoom.value
  const maxPanY = excessY / zoom.value

  panX.value = Math.max(-maxPanX, Math.min(maxPanX, panX.value))
  panY.value = Math.max(-maxPanY, Math.min(maxPanY, panY.value))
}

// Drag/pan functions
function startDrag(e: MouseEvent) {
  if (!canDrag.value) return

  isDragging.value = true
  dragStartX.value = e.clientX
  dragStartY.value = e.clientY
  dragStartPanX.value = panX.value
  dragStartPanY.value = panY.value
}

function onDrag(e: MouseEvent) {
  if (!isDragging.value) return

  const dx = (e.clientX - dragStartX.value) / zoom.value
  const dy = (e.clientY - dragStartY.value) / zoom.value

  panX.value = dragStartPanX.value + dx
  panY.value = dragStartPanY.value + dy
  clampPan()
}

function endDrag() {
  isDragging.value = false
}

// Navigation
function prev() {
  if (currentIndex.value > 0) {
    currentIndex.value--
  }
}

function next() {
  if (currentIndex.value < props.files.length - 1) {
    currentIndex.value++
  }
}

function handleOverlayClick(e: MouseEvent) {
  if (e.target === e.currentTarget) {
    emit('close')
  }
}

// Download file via fetch to handle cross-origin URLs
async function downloadCurrentFile() {
  if (!currentFile.value) return

  try {
    const response = await fetch(currentFile.value.url)
    const blob = await response.blob()
    const url = URL.createObjectURL(blob)

    const a = document.createElement('a')
    a.href = url
    a.download = currentFile.value.filename
    document.body.appendChild(a)
    a.click()
    document.body.removeChild(a)

    URL.revokeObjectURL(url)
  } catch (e) {
    console.error('Download failed:', e)
    // Fallback: open in new tab
    window.open(currentFile.value.url, '_blank')
  }
}

function handleWheel(e: WheelEvent) {
  if (currentFileType.value !== 'image' && currentFileType.value !== 'pdf') return

  // Ctrl+wheel or pinch for zoom
  if (e.ctrlKey || e.metaKey) {
    e.preventDefault()
    handleZoomWheel(e.deltaY, e.clientX, e.clientY)
  } else {
    // Two-finger trackpad scroll for panning
    e.preventDefault()
    setInteracting(true)
    panX.value -= e.deltaX / zoom.value
    panY.value -= e.deltaY / zoom.value
    clampPan()
  }
}

// File type helpers
function getFileTypeLabel(contentType?: string): string {
  const labels = t.value.fileViewer.fileTypes
  if (!contentType) return labels.file

  const typeMap: Record<string, string> = {
    'application/msword': labels.word,
    'application/vnd.openxmlformats-officedocument.wordprocessingml.document': labels.word,
    'application/vnd.ms-excel': labels.excel,
    'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet': labels.excel,
    'application/vnd.ms-powerpoint': labels.powerpoint,
    'application/vnd.openxmlformats-officedocument.presentationml.presentation': labels.powerpoint,
    'application/zip': labels.zip,
    'application/x-rar-compressed': labels.rar,
    'application/vnd.rar': labels.rar,
    'application/x-7z-compressed': labels.sevenZip,
    'application/gzip': labels.gzip,
    'text/plain': labels.text,
    'text/csv': labels.csv,
    'application/json': labels.json,
    'application/xml': labels.xml,
    'text/xml': labels.xml,
    'video/mp4': labels.video,
    'video/webm': labels.video,
    'video/ogg': labels.video,
    'video/quicktime': labels.video,
    'audio/mpeg': labels.audio,
    'audio/wav': labels.audio,
    'audio/ogg': labels.audio,
    'audio/mp4': labels.audio,
  }

  return typeMap[contentType] || contentType.split('/')[1]?.toUpperCase() || labels.file
}

// File icons as functional components
const IconDocument: FunctionalComponent = () => h('svg', {
  width: 64, height: 64, viewBox: '0 0 24 24', fill: 'none', stroke: 'currentColor', 'stroke-width': 1.5
}, [
  h('path', { d: 'M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z' }),
  h('path', { d: 'M14 2v6h6M16 13H8M16 17H8M10 9H8' })
])

const IconSpreadsheet: FunctionalComponent = () => h('svg', {
  width: 64, height: 64, viewBox: '0 0 24 24', fill: 'none', stroke: 'currentColor', 'stroke-width': 1.5
}, [
  h('path', { d: 'M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z' }),
  h('path', { d: 'M14 2v6h6M8 13h8M8 17h8M8 9h2' })
])

const IconArchive: FunctionalComponent = () => h('svg', {
  width: 64, height: 64, viewBox: '0 0 24 24', fill: 'none', stroke: 'currentColor', 'stroke-width': 1.5
}, [
  h('path', { d: 'M21 8v13H3V8M1 3h22v5H1zM10 12h4' })
])

const IconVideo: FunctionalComponent = () => h('svg', {
  width: 64, height: 64, viewBox: '0 0 24 24', fill: 'none', stroke: 'currentColor', 'stroke-width': 1.5
}, [
  h('rect', { x: 2, y: 2, width: 20, height: 20, rx: 2.18, ry: 2.18 }),
  h('path', { d: 'M10 8l6 4-6 4V8z' })
])

const IconAudio: FunctionalComponent = () => h('svg', {
  width: 64, height: 64, viewBox: '0 0 24 24', fill: 'none', stroke: 'currentColor', 'stroke-width': 1.5
}, [
  h('path', { d: 'M9 18V5l12-2v13' }),
  h('circle', { cx: 6, cy: 18, r: 3 }),
  h('circle', { cx: 18, cy: 16, r: 3 })
])

const IconFile: FunctionalComponent = () => h('svg', {
  width: 64, height: 64, viewBox: '0 0 24 24', fill: 'none', stroke: 'currentColor', 'stroke-width': 1.5
}, [
  h('path', { d: 'M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z' }),
  h('path', { d: 'M14 2v6h6' })
])

function getFileIcon(contentType?: string): FunctionalComponent {
  if (!contentType) return IconFile

  if (contentType.includes('word') || contentType.includes('document')) return IconDocument
  if (contentType.includes('excel') || contentType.includes('spreadsheet')) return IconSpreadsheet
  if (contentType.includes('zip') || contentType.includes('rar') || contentType.includes('7z') || contentType.includes('gzip')) return IconArchive
  if (contentType.startsWith('video/')) return IconVideo
  if (contentType.startsWith('audio/')) return IconAudio

  return IconFile
}

// Keyboard handling
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
    case '+':
    case '=':
      if (currentFileType.value === 'image' || currentFileType.value === 'pdf') zoomIn()
      break
    case '-':
      if (currentFileType.value === 'image' || currentFileType.value === 'pdf') zoomOut()
      break
    case '0':
      resetZoom()
      break
  }
}

function handleGlobalWheel(e: WheelEvent) {
  if (!props.show) return
  // Only handle Ctrl+wheel for zoom globally (content wheel handles pan)
  if ((e.ctrlKey || e.metaKey) && (currentFileType.value === 'image' || currentFileType.value === 'pdf')) {
    e.preventDefault()
    handleZoomWheel(e.deltaY, e.clientX, e.clientY)
  }
}

onMounted(() => {
  document.addEventListener('keydown', handleKeydown)
  document.addEventListener('wheel', handleGlobalWheel, { passive: false })
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeydown)
  document.removeEventListener('wheel', handleGlobalWheel)
  cancelAllRenderTasks()
})
</script>

<style scoped>
.viewer-overlay {
  position: fixed;
  inset: 0;
  z-index: 10000;
  background: rgba(0, 0, 0, 0.9);
  display: flex;
  align-items: center;
  justify-content: center;
}

.viewer-container {
  position: relative;
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
}

.viewer-close {
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

.viewer-close:hover {
  background: rgba(255, 255, 255, 0.2);
}

.viewer-nav {
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

.viewer-nav:disabled {
  opacity: 0.3;
  cursor: not-allowed;
}

.viewer-nav:not(:disabled):hover {
  background: rgba(255, 255, 255, 0.2);
}

.viewer-nav--prev {
  left: 16px;
}

.viewer-nav--next {
  right: 16px;
}

/* Content area */
.viewer-content {
  position: relative;
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
  padding: 80px 120px 100px;
  user-select: none;
  opacity: 0;
  transition: opacity 0.5s ease-out;
}

.viewer-content--ready {
  opacity: 1;
}

.viewer-content--draggable {
  cursor: grab;
}

.viewer-content--draggable:active {
  cursor: grabbing;
}

/* Image */
.viewer-image-wrapper {
  display: flex;
  align-items: center;
  justify-content: center;
  transform-origin: center center;
  max-width: 60vw;
  max-height: 100%;
  opacity: 0;
  transition: opacity 0.5s ease-out;
}

.viewer-content--ready .viewer-image-wrapper {
  opacity: 1;
}

.viewer-image {
  max-width: 100%;
  max-height: 100%;
  object-fit: contain;
  pointer-events: none;
}

/* PDF */
.viewer-pdf-wrapper {
  display: flex;
  justify-content: center;
  transform-origin: center center;
  max-width: 60vw;
  opacity: 0;
  transition: opacity 0.8s ease-out;
}

.viewer-content--ready .viewer-pdf-wrapper {
  opacity: 1;
}

.pdf-pages {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
}

.pdf-page {
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.4);
  background: white;
}

/* File preview (unsupported types) */
.viewer-file-preview {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 16px;
  color: rgba(255, 255, 255, 0.7);
  text-align: center;
  padding: 40px;
}

.viewer-file-icon {
  color: rgba(255, 255, 255, 0.5);
}

.viewer-file-name {
  font-size: 18px;
  font-weight: 500;
  color: white;
  max-width: 400px;
  word-break: break-word;
}

.viewer-file-name--clickable {
  text-decoration: none;
  cursor: pointer;
  transition: color 0.2s;
}

.viewer-file-name--clickable:hover {
  color: #4fc3f7;
  text-decoration: underline;
}

.viewer-file-type {
  font-size: 14px;
  color: rgba(255, 255, 255, 0.5);
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

/* Footer */
.viewer-footer {
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

.viewer-filename {
  flex: 1;
  font-size: 14px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.viewer-counter {
  font-size: 14px;
  color: rgba(255, 255, 255, 0.7);
}

.viewer-pages-count {
  font-size: 14px;
  color: rgba(255, 255, 255, 0.7);
}

.viewer-zoom {
  display: flex;
  align-items: center;
  gap: 4px;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 18px;
  padding: 4px;
}

.viewer-zoom-btn {
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

.viewer-zoom-btn:hover:not(:disabled) {
  background: rgba(255, 255, 255, 0.15);
  color: white;
}

.viewer-zoom-btn:disabled {
  opacity: 0.3;
  cursor: not-allowed;
}

.viewer-zoom-level {
  font-size: 12px;
  color: rgba(255, 255, 255, 0.8);
  min-width: 40px;
  text-align: center;
}

.viewer-download {
  width: 36px;
  height: 36px;
  border: none;
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.1);
  color: white;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background 0.2s;
  cursor: pointer;
}

.viewer-download:hover {
  background: rgba(255, 255, 255, 0.2);
}
</style>
