/**
 * MTChat REST API Client
 *
 * v3 Architecture: Object-bound dialogs with scope-based access
 */

import type {
  Dialog,
  DialogListItem,
  DialogParticipant,
  Message,
  ApiResponse,
  PaginationOptions,
  DialogListType,
  ScopeConfig,
  PresignUploadResponse,
  AttachmentInput,
} from '../types'

/**
 * REST API client for MTChat
 */
export class MTChatApi {
  private baseUrl: string
  private userId: string
  private scopeConfig: ScopeConfig

  constructor(baseUrl: string, userId: string, scopeConfig: ScopeConfig) {
    this.baseUrl = baseUrl.replace(/\/$/, '')
    this.userId = userId
    this.scopeConfig = scopeConfig
  }

  /**
   * Encode scope config as base64 JSON for X-Scope-Config header
   */
  private encodeScopeConfig(): string {
    const json = JSON.stringify(this.scopeConfig)
    return btoa(json)
  }

  /**
   * Build common headers for requests
   */
  private getHeaders(): HeadersInit {
    return {
      'Content-Type': 'application/json',
      'X-Scope-Config': this.encodeScopeConfig(),
    }
  }

  /**
   * Build URL with user_id query parameter
   */
  private buildUrl(path: string, params: Record<string, string> = {}): string {
    const url = new URL(`${this.baseUrl}${path}`)
    url.searchParams.set('user_id', this.userId)
    for (const [key, value] of Object.entries(params)) {
      if (value !== undefined && value !== null) {
        url.searchParams.set(key, value)
      }
    }
    return url.toString()
  }

  /**
   * Make API request
   */
  private async request<T>(
    method: string,
    path: string,
    options: {
      params?: Record<string, string>
      body?: unknown
    } = {}
  ): Promise<T> {
    const url = this.buildUrl(path, options.params)

    const response = await fetch(url, {
      method,
      headers: this.getHeaders(),
      body: options.body ? JSON.stringify(options.body) : undefined,
    })

    if (!response.ok) {
      const error = await response.json().catch(() => ({ error: { message: response.statusText } }))
      throw new Error(error.error?.message || `HTTP ${response.status}`)
    }

    // Handle 204 No Content
    if (response.status === 204) {
      return undefined as T
    }

    return response.json()
  }

  // ============ Health ============

  /**
   * Check API health
   */
  async healthCheck(): Promise<{ status: string; version: string }> {
    const response = await fetch(`${this.baseUrl}/health`)
    return response.json()
  }

  // ============ Dialogs ============

  /**
   * List dialogs
   * @param type - 'participating' (my chats) or 'available' (can join)
   */
  async getDialogs(type: DialogListType = 'participating'): Promise<DialogListItem[]> {
    const response = await this.request<ApiResponse<DialogListItem[]>>(
      'GET',
      '/api/v1/dialogs',
      { params: { type } }
    )
    return response.data
  }

  /**
   * Get dialogs user is participating in
   */
  async getParticipatingDialogs(): Promise<DialogListItem[]> {
    return this.getDialogs('participating')
  }

  /**
   * Get dialogs user can join (based on scope)
   */
  async getAvailableDialogs(): Promise<DialogListItem[]> {
    return this.getDialogs('available')
  }

  /**
   * Get dialog by ID
   */
  async getDialog(dialogId: string): Promise<Dialog> {
    const response = await this.request<ApiResponse<Dialog>>(
      'GET',
      `/api/v1/dialogs/${dialogId}`
    )
    return response.data
  }

  /**
   * Get dialog by business object (for inline mode)
   * Returns null if no dialog exists for this object
   */
  async getDialogByObject(objectType: string, objectId: string): Promise<DialogListItem | null> {
    const response = await this.request<ApiResponse<DialogListItem | null>>(
      'GET',
      `/api/v1/dialogs/by-object/${objectType}/${objectId}`
    )
    return response.data
  }

  /**
   * Join a dialog (become participant)
   */
  async joinDialog(dialogId: string): Promise<{ status: string; dialog_id: string }> {
    return this.request<{ status: string; dialog_id: string }>(
      'POST',
      `/api/v1/dialogs/${dialogId}/join`
    )
  }

  /**
   * Leave a dialog
   */
  async leaveDialog(dialogId: string): Promise<void> {
    await this.request<void>('POST', `/api/v1/dialogs/${dialogId}/leave`)
  }

  /**
   * Get dialog participants
   */
  async getParticipants(dialogId: string): Promise<DialogParticipant[]> {
    const response = await this.request<ApiResponse<DialogParticipant[]>>(
      'GET',
      `/api/v1/dialogs/${dialogId}/participants`
    )
    return response.data
  }

  // ============ Messages ============

  /**
   * Get messages in a dialog
   */
  async getMessages(dialogId: string, options?: PaginationOptions): Promise<Message[]> {
    const params: Record<string, string> = {}
    if (options?.limit) params.limit = String(options.limit)
    if (options?.before) params.before = options.before

    const response = await this.request<ApiResponse<Message[]>>(
      'GET',
      `/api/v1/dialogs/${dialogId}/messages`,
      { params }
    )
    return response.data
  }

  /**
   * Get a specific message
   */
  async getMessage(dialogId: string, messageId: string): Promise<Message> {
    const response = await this.request<ApiResponse<Message>>(
      'GET',
      `/api/v1/dialogs/${dialogId}/messages/${messageId}`
    )
    return response.data
  }

  /**
   * Send a message
   */
  async sendMessage(
    dialogId: string,
    content: string,
    options?: { replyTo?: string; attachments?: AttachmentInput[] }
  ): Promise<Message> {
    const response = await this.request<ApiResponse<Message>>(
      'POST',
      `/api/v1/dialogs/${dialogId}/messages`,
      {
        body: {
          content,
          reply_to: options?.replyTo,
          attachments: options?.attachments || [],
        },
      }
    )
    return response.data
  }

  // ============ Upload ============

  /**
   * Get presigned URL for file upload
   */
  async getPresignedUploadUrl(
    dialogId: string,
    filename: string,
    contentType: string,
    size: number
  ): Promise<PresignUploadResponse> {
    const response = await this.request<ApiResponse<PresignUploadResponse>>(
      'POST',
      '/api/v1/upload/presign',
      {
        body: {
          dialog_id: dialogId,
          filename,
          content_type: contentType,
          size,
        },
      }
    )
    return response.data
  }

  /**
   * Get presigned download URL for attachment
   */
  async getAttachmentUrl(attachmentId: string): Promise<{ url: string; thumbnail_url?: string; expires_in: number }> {
    const response = await this.request<ApiResponse<{ url: string; thumbnail_url?: string; expires_in: number }>>(
      'GET',
      `/api/v1/attachments/${attachmentId}/url`
    )
    return response.data
  }

  /**
   * Upload file to S3 using presigned URL
   * Returns progress updates via callback
   */
  uploadFile(
    uploadUrl: string,
    file: File,
    onProgress?: (progress: number) => void
  ): Promise<void> {
    return new Promise((resolve, reject) => {
      const xhr = new XMLHttpRequest()

      xhr.upload.addEventListener('progress', (e) => {
        if (e.lengthComputable && onProgress) {
          onProgress(Math.round((e.loaded / e.total) * 100))
        }
      })

      xhr.addEventListener('load', () => {
        if (xhr.status >= 200 && xhr.status < 300) {
          resolve()
        } else {
          reject(new Error(`Upload failed: ${xhr.status}`))
        }
      })

      xhr.addEventListener('error', () => reject(new Error('Upload failed')))
      xhr.addEventListener('abort', () => reject(new Error('Upload aborted')))

      xhr.open('PUT', uploadUrl)
      xhr.setRequestHeader('Content-Type', file.type)
      xhr.send(file)
    })
  }
}
