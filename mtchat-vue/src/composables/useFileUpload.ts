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

      pendingAttachments.value.push(pending)

      // Start upload
      uploadFile(pending)
    }
  }

  /**
   * Upload a single file
   */
  async function uploadFile(pending: PendingAttachment): Promise<void> {
    // Find the item in the array (for reactivity)
    const item = pendingAttachments.value.find((a) => a.id === pending.id)
    if (!item) return

    if (!dialogId.value) {
      item.status = 'error'
      item.error = 'No dialog selected'
      return
    }

    try {
      item.status = 'uploading'
      item.progress = 0

      // Get presigned URL
      const { upload_url, s3_key } = await api.getPresignedUploadUrl(
        dialogId.value,
        item.filename,
        item.contentType,
        item.size
      )

      // Upload to S3
      await api.uploadFile(upload_url, item.file, (progress) => {
        item.progress = progress
      })

      item.s3Key = s3_key
      item.status = 'uploaded'
      item.progress = 100
    } catch (error) {
      item.status = 'error'
      item.error = error instanceof Error ? error.message : 'Upload failed'
      onError?.(item.error)
    }
  }

  /**
   * Remove a pending attachment
   */
  function removeAttachment(id: string): void {
    const index = pendingAttachments.value.findIndex((a) => a.id === id)
    if (index !== -1) {
      const attachment = pendingAttachments.value[index]
      // Revoke blob URL if exists
      if (attachment.previewUrl) {
        URL.revokeObjectURL(attachment.previewUrl)
      }
      pendingAttachments.value.splice(index, 1)
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
      pending.error = undefined
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
