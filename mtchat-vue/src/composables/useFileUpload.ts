/**
 * useFileUpload composable
 *
 * Manages file uploads with progress tracking and validation
 */

import { ref, computed, type Ref, type ComputedRef } from 'vue'
import type { PendingAttachment, AttachmentInput } from '../types'
import { ATTACHMENT_LIMITS, isAllowedFileType, isValidFileSize } from '../types'
import type { MTChatApi } from '../sdk/api'

export interface UseFileUploadOptions {
  /** Dialog ID for upload */
  dialogId: Ref<string | undefined> | ComputedRef<string | undefined>
  /** API client instance */
  api: MTChatApi
  /** Callback on upload error */
  onError?: (error: string) => void
}

export interface UseFileUploadReturn {
  /** List of pending attachments */
  pendingAttachments: Ref<PendingAttachment[]>
  /** Whether any file is currently uploading */
  isUploading: Ref<boolean>
  /** Whether all files are uploaded and ready */
  isReady: Ref<boolean>
  /** Whether at least one file is uploaded */
  hasUploaded: Ref<boolean>
  /** Add files to upload queue */
  addFiles: (files: FileList | File[]) => Promise<void>
  /** Remove a pending attachment */
  removeAttachment: (id: string) => void
  /** Get uploaded attachments for sending */
  getUploadedAttachments: () => AttachmentInput[]
  /** Clear all pending attachments */
  clearAll: () => void
  /** Retry failed upload */
  retryUpload: (id: string) => Promise<void>
}

export function useFileUpload(options: UseFileUploadOptions): UseFileUploadReturn {
  const { dialogId, api, onError } = options

  const pendingAttachments = ref<PendingAttachment[]>([])

  const isUploading = computed(() =>
    pendingAttachments.value.some((a) => a.status === 'uploading')
  )

  const isReady = computed(() =>
    pendingAttachments.value.length > 0 &&
    pendingAttachments.value.every((a) => a.status === 'uploaded')
  )

  const hasUploaded = computed(() =>
    pendingAttachments.value.some((a) => a.status === 'uploaded')
  )

  /**
   * Add files to upload queue and start uploading
   */
  async function addFiles(files: FileList | File[]): Promise<void> {
    const fileArray = Array.from(files)

    // Check total count
    const totalCount = pendingAttachments.value.length + fileArray.length
    if (totalCount > ATTACHMENT_LIMITS.MAX_ATTACHMENTS_PER_MESSAGE) {
      const error = `Maximum ${ATTACHMENT_LIMITS.MAX_ATTACHMENTS_PER_MESSAGE} files per message`
      onError?.(error)
      return
    }

    for (const file of fileArray) {
      // Validate type
      if (!isAllowedFileType(file.type)) {
        onError?.(`File type "${file.type}" is not allowed`)
        continue
      }

      // Validate size
      if (!isValidFileSize(file.size)) {
        onError?.(`File "${file.name}" is too large (max ${ATTACHMENT_LIMITS.MAX_FILE_SIZE / 1024 / 1024}MB)`)
        continue
      }

      // Create pending attachment
      const pending: PendingAttachment = {
        id: crypto.randomUUID(),
        file,
        filename: file.name,
        contentType: file.type,
        size: file.size,
        progress: 0,
        status: 'pending',
        previewUrl: file.type.startsWith('image/') ? URL.createObjectURL(file) : undefined,
      }

      pendingAttachments.value = [...pendingAttachments.value, pending]

      // Start upload
      uploadFile(pending)
    }
  }

  /**
   * Immutable update helper: update a single item by ID
   */
  function updateItem(id: string, updates: Partial<PendingAttachment>): void {
    pendingAttachments.value = pendingAttachments.value.map((a) =>
      a.id === id ? { ...a, ...updates } : a
    )
  }

  /**
   * Upload a single file
   */
  async function uploadFile(pending: PendingAttachment): Promise<void> {
    if (!pendingAttachments.value.some((a) => a.id === pending.id)) return

    if (!dialogId.value) {
      updateItem(pending.id, { status: 'error', error: 'No dialog selected' })
      return
    }

    try {
      updateItem(pending.id, { status: 'uploading', progress: 0 })

      // Get presigned URL
      const { upload_url, s3_key } = await api.getPresignedUploadUrl(
        dialogId.value,
        pending.filename,
        pending.contentType,
        pending.size
      )

      // Upload to S3
      await api.uploadFile(upload_url, pending.file, (progress) => {
        updateItem(pending.id, { progress })
      })

      updateItem(pending.id, { s3Key: s3_key, status: 'uploaded', progress: 100 })
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : 'Upload failed'
      updateItem(pending.id, { status: 'error', error: errorMsg })
      onError?.(errorMsg)
    }
  }

  /**
   * Remove a pending attachment
   */
  function removeAttachment(id: string): void {
    const attachment = pendingAttachments.value.find((a) => a.id === id)
    if (attachment) {
      // Revoke blob URL if exists
      if (attachment.previewUrl) {
        URL.revokeObjectURL(attachment.previewUrl)
      }
      pendingAttachments.value = pendingAttachments.value.filter((a) => a.id !== id)
    }
  }

  /**
   * Get uploaded attachments for sending message
   */
  function getUploadedAttachments(): AttachmentInput[] {
    return pendingAttachments.value
      .filter((a) => a.status === 'uploaded' && a.s3Key)
      .map((a) => ({
        s3_key: a.s3Key!,
        filename: a.filename,
        content_type: a.contentType,
        size: a.size,
      }))
  }

  /**
   * Clear all pending attachments
   */
  function clearAll(): void {
    // Revoke all blob URLs
    for (const attachment of pendingAttachments.value) {
      if (attachment.previewUrl) {
        URL.revokeObjectURL(attachment.previewUrl)
      }
    }
    pendingAttachments.value = []
  }

  /**
   * Retry a failed upload
   */
  async function retryUpload(id: string): Promise<void> {
    const pending = pendingAttachments.value.find((a) => a.id === id)
    if (pending && pending.status === 'error') {
      updateItem(id, { error: undefined, status: 'pending' })
      await uploadFile(pending)
    }
  }

  return {
    pendingAttachments,
    isUploading,
    isReady,
    hasUploaded,
    addFiles,
    removeAttachment,
    getUploadedAttachments,
    clearAll,
    retryUpload,
  }
}
