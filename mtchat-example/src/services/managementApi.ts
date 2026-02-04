import type { AccessScope } from '../types'

/**
 * Management API request/response types
 */
export interface CreateDialogRequest {
  object_id: string
  object_type: string
  title?: string
  participants?: string[]
  access_scopes?: {
    tenant_uid: string
    scope_level1: string[]
    scope_level2: string[]
  }[]
}

export interface DialogResponse {
  id: string
  object_id: string
  object_type: string
  title?: string
  created_by?: string
  created_at: string
}

export interface ApiError {
  error: {
    code: string
    message: string
  }
}

/**
 * Management API Client
 *
 * Used to create/delete dialogs via admin token.
 * In production, this would be called from the host application's backend.
 */
export class ManagementApi {
  private baseUrl: string
  private adminToken: string

  constructor(baseUrl: string, adminToken: string) {
    this.baseUrl = baseUrl.replace(/\/$/, '')
    this.adminToken = adminToken
  }

  setAdminToken(token: string): void {
    this.adminToken = token
  }

  private getHeaders(): HeadersInit {
    return {
      'Content-Type': 'application/json',
      Authorization: `Bearer ${this.adminToken}`,
    }
  }

  private async request<T>(
    method: string,
    path: string,
    body?: unknown
  ): Promise<T> {
    const response = await fetch(`${this.baseUrl}${path}`, {
      method,
      headers: this.getHeaders(),
      body: body ? JSON.stringify(body) : undefined,
    })

    if (!response.ok) {
      const error = (await response.json().catch(() => ({
        error: { message: response.statusText },
      }))) as ApiError
      throw new Error(error.error?.message || `HTTP ${response.status}`)
    }

    // Handle 204 No Content
    if (response.status === 204) {
      return undefined as T
    }

    const data = await response.json()
    return data.data ?? data
  }

  /**
   * Create a new dialog
   */
  async createDialog(request: CreateDialogRequest): Promise<DialogResponse> {
    return this.request<DialogResponse>(
      'POST',
      '/api/v1/management/dialogs',
      request
    )
  }

  /**
   * Get a dialog by ID
   */
  async getDialog(id: string): Promise<DialogResponse> {
    return this.request<DialogResponse>('GET', `/api/v1/management/dialogs/${id}`)
  }

  /**
   * Delete a dialog
   */
  async deleteDialog(id: string): Promise<void> {
    await this.request<void>('DELETE', `/api/v1/management/dialogs/${id}`)
  }

  /**
   * Add a participant to a dialog
   */
  async addParticipant(
    dialogId: string,
    userId: string,
    notifications?: boolean
  ): Promise<void> {
    await this.request<void>('POST', `/api/v1/management/dialogs/${dialogId}/participants`, {
      user_id: userId,
      notifications_enabled: notifications ?? true,
    })
  }

  /**
   * Remove a participant from a dialog
   */
  async removeParticipant(dialogId: string, userId: string): Promise<void> {
    await this.request<void>(
      'DELETE',
      `/api/v1/management/dialogs/${dialogId}/participants/${userId}`
    )
  }

  /**
   * Add an access scope to a dialog
   */
  async addAccessScope(
    dialogId: string,
    scope: {
      tenant_uid: string
      scope_level1: string[]
      scope_level2: string[]
    }
  ): Promise<void> {
    await this.request<void>('POST', `/api/v1/management/dialogs/${dialogId}/scopes`, scope)
  }
}

/**
 * Create a Management API instance
 */
export function createManagementApi(baseUrl: string, adminToken: string): ManagementApi {
  return new ManagementApi(baseUrl, adminToken)
}
