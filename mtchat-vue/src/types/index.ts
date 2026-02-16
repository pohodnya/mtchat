/**
 * MTChat Vue SDK Types
 *
 * v3 Architecture: Object-bound dialogs with scope-based access
 */

// ============ Core Entities ============

/**
 * Dialog (chat) bound to a business object
 */
export interface Dialog {
  id: string
  /** Business object ID (tender, order, route, etc.) */
  object_id: string
  /** Business object type */
  object_type: string
  /** Optional dialog title */
  title?: string
  /** URL to the object page in the host system */
  object_url?: string
  /** User who created the dialog */
  created_by?: string
  created_at: string
}

/**
 * Dialog with additional metadata for list display
 */
export interface DialogListItem extends Dialog {
  /** Number of participants */
  participants_count: number
  /** Whether current user is a participant */
  i_am_participant?: boolean
  /** Whether current user can join */
  can_join?: boolean
  /** Unread messages count */
  unread_count?: number
  /** Whether dialog is archived for current user */
  is_archived?: boolean
  /** Whether dialog is pinned for current user */
  is_pinned?: boolean
  /** Whether notifications are enabled for current user */
  notifications_enabled?: boolean
  /** Timestamp of the last message in this dialog */
  last_message_at?: string
}

/**
 * Dialog participant
 */
export interface DialogParticipant {
  dialog_id: string
  user_id: string
  joined_at: string
  /** How user joined: 'creator', 'participant', 'joined' */
  joined_as: 'creator' | 'participant' | 'joined'
  notifications_enabled: boolean
  last_read_message_id?: string
  /** Number of unread messages */
  unread_count: number
  /** Display name (full name, initials, or anonymous) */
  display_name?: string
  /** Company/organization name */
  company?: string
  /** Contact email (optional, can be hidden) */
  email?: string
  /** Contact phone (optional, can be hidden) */
  phone?: string
  /** Whether user is currently online */
  is_online?: boolean
}

/**
 * Profile information for joining a dialog
 */
export interface ParticipantProfile {
  display_name: string
  company?: string
  email?: string
  phone?: string
}

/**
 * Request body for joining a dialog
 */
export interface JoinDialogRequest {
  display_name: string
  company: string
  email?: string
  phone?: string
}

/**
 * Access scope for potential participants
 */
export interface DialogAccessScope {
  id: string
  dialog_id: string
  tenant_uid: string
  scope_level1: string[]
  scope_level2: string[]
  created_at: string
}

/**
 * Message type
 */
export type MessageType = 'user' | 'system'

/**
 * System message event types
 */
export type SystemMessageEvent = 'chat_created' | 'participant_joined' | 'participant_left'

/**
 * System message content structure (parsed from JSON)
 */
export interface SystemMessageContent {
  event: SystemMessageEvent
  /** Participants list for chat_created event */
  participants?: Array<{ name: string; company?: string }>
  /** Participant name for joined/left events */
  name?: string
  /** Participant company for joined event */
  company?: string
}

/**
 * Chat message
 */
export interface Message {
  id: string
  dialog_id: string
  /** Sender ID. Null for system messages. */
  sender_id: string | null
  content: string
  sent_at: string
  last_edited_at?: string
  reply_to_id?: string
  /** Attachments with presigned URLs */
  attachments?: Attachment[]
  /** Message type: 'user' or 'system' (default: 'user') */
  message_type?: MessageType
}

// ============ Attachments ============

/**
 * File attachment on a message
 */
export interface Attachment {
  id: string
  filename: string
  content_type: string
  size: number
  /** Image width in pixels (images only) */
  width?: number
  /** Image height in pixels (images only) */
  height?: number
  /** Presigned download URL */
  url: string
  /** Presigned thumbnail URL (images only) */
  thumbnail_url?: string
}

/**
 * Attachment type for display purposes
 */
export type AttachmentType = 'image' | 'pdf' | 'file'

/**
 * Get attachment type from content type
 */
export function getAttachmentType(contentType: string): AttachmentType {
  if (contentType.startsWith('image/')) return 'image'
  if (contentType === 'application/pdf') return 'pdf'
  return 'file'
}

/**
 * Pending attachment (before message is sent)
 */
export interface PendingAttachment {
  /** Temporary client-side ID */
  id: string
  file: File
  filename: string
  contentType: string
  size: number
  /** Upload progress 0-100 */
  progress: number
  status: 'pending' | 'uploading' | 'uploaded' | 'error'
  /** S3 key after upload */
  s3Key?: string
  /** Local blob URL for preview */
  previewUrl?: string
  /** Error message */
  error?: string
}

/**
 * Presigned upload URL response
 */
export interface PresignUploadResponse {
  upload_url: string
  s3_key: string
  expires_in: number
}

/**
 * Attachment input for sending message
 */
export interface AttachmentInput {
  s3_key: string
  filename: string
  content_type: string
  size: number
}

/**
 * Attachment limits
 */
export const ATTACHMENT_LIMITS = {
  MAX_FILE_SIZE: 100 * 1024 * 1024, // 100MB
  MAX_ATTACHMENTS_PER_MESSAGE: 10,
  ALLOWED_TYPES: [
    // Images
    'image/jpeg',
    'image/png',
    'image/gif',
    'image/webp',
    'image/svg+xml',
    'image/bmp',
    'image/tiff',
    // Documents
    'application/pdf',
    'application/msword', // .doc
    'application/vnd.openxmlformats-officedocument.wordprocessingml.document', // .docx
    'application/vnd.ms-excel', // .xls
    'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet', // .xlsx
    'application/vnd.ms-powerpoint', // .ppt
    'application/vnd.openxmlformats-officedocument.presentationml.presentation', // .pptx
    'application/vnd.oasis.opendocument.text', // .odt
    'application/vnd.oasis.opendocument.spreadsheet', // .ods
    'application/vnd.oasis.opendocument.presentation', // .odp
    'application/rtf',
    // Text
    'text/plain',
    'text/csv',
    'text/markdown',
    'text/html',
    'text/xml',
    'application/json',
    // Archives
    'application/zip',
    'application/x-rar-compressed',
    'application/vnd.rar',
    'application/x-7z-compressed',
    'application/gzip',
    'application/x-tar',
    // Audio
    'audio/mpeg',
    'audio/wav',
    'audio/ogg',
    'audio/mp4',
    // Video
    'video/mp4',
    'video/webm',
    'video/ogg',
    'video/quicktime',
  ],
} as const

/**
 * Check if file type is allowed
 * Allows all types from ALLOWED_TYPES list, or any type if list checking is disabled
 */
export function isAllowedFileType(contentType: string): boolean {
  // Allow empty content type (browser couldn't detect)
  if (!contentType) return true
  // Check against allowed list
  return ATTACHMENT_LIMITS.ALLOWED_TYPES.includes(contentType as typeof ATTACHMENT_LIMITS.ALLOWED_TYPES[number])
}

/**
 * Check if file size is valid
 */
export function isValidFileSize(size: number): boolean {
  return size > 0 && size <= ATTACHMENT_LIMITS.MAX_FILE_SIZE
}

/**
 * Format file size for display
 */
export function formatFileSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}

/**
 * Get icon name for file type (for use with Icon component)
 */
export function getFileIconName(contentType?: string): string {
  if (!contentType) return 'file'

  // PDF
  if (contentType === 'application/pdf') return 'file-pdf'

  // Images
  if (contentType.startsWith('image/')) return 'image'

  // Spreadsheets (Excel, ODS, CSV) - check before documents due to 'spreadsheetml.document' in xlsx MIME
  if (
    contentType.includes('excel') ||
    contentType.includes('spreadsheet') ||
    contentType === 'text/csv'
  ) return 'file-spreadsheet'

  // Presentations (PowerPoint, ODP) - check before documents due to 'presentation' in MIME
  if (
    contentType.includes('powerpoint') ||
    contentType.includes('presentation')
  ) return 'file-document'

  // Documents (Word, ODT, RTF)
  if (
    contentType.includes('word') ||
    contentType.includes('wordprocessing') ||
    contentType.includes('opendocument.text') ||
    contentType === 'application/rtf'
  ) return 'file-document'

  // Archives
  if (
    contentType.includes('zip') ||
    contentType.includes('rar') ||
    contentType.includes('7z') ||
    contentType.includes('gzip') ||
    contentType.includes('tar')
  ) return 'file-archive'

  // Video
  if (contentType.startsWith('video/')) return 'file-video'

  // Audio
  if (contentType.startsWith('audio/')) return 'file-audio'

  // Text/Code files
  if (
    contentType.startsWith('text/') ||
    contentType === 'application/json' ||
    contentType === 'application/xml'
  ) return 'file-document'

  return 'file'
}

// ============ API Request/Response ============

/**
 * Scope configuration for current user
 * Passed to API via X-Scope-Config header (base64 JSON)
 */
export interface ScopeConfig {
  /** User's tenant/organization ID */
  tenant_uid: string
  /** First level scope (e.g., departments) */
  scope_level1: string[]
  /** Second level scope (e.g., roles/permissions) */
  scope_level2: string[]
}

/**
 * API response wrapper
 */
export interface ApiResponse<T> {
  data: T
}

/**
 * Pagination options for message loading
 */
export interface PaginationOptions {
  limit?: number
  before?: string
  /** Load messages after this message ID (for loading newer messages) */
  after?: string
  /** Load messages centered around this message ID (for jumping to replies) */
  around?: string
}

/**
 * Messages response with unread tracking
 */
export interface MessagesResponse {
  messages: Message[]
  /** ID of the first unread message (for divider positioning) */
  first_unread_message_id?: string
  /** Whether there are more messages before the loaded set */
  has_more_before?: boolean
  /** Whether there are more messages after the loaded set */
  has_more_after?: boolean
}

/**
 * Dialog list filter type
 */
export type DialogListType = 'participating' | 'available'

// ============ WebSocket Events ============

/**
 * WebSocket event types from server
 */
export type WsEventType =
  | 'connected'
  | 'message.new'
  | 'message.read'
  | 'message.edited'
  | 'message.deleted'
  | 'participant.joined'
  | 'participant.left'
  | 'dialog.archived'
  | 'dialog.unarchived'
  | 'presence.update'
  | 'typing'
  | 'pong'
  | 'error'

/**
 * WebSocket event from server
 */
export interface WsEvent {
  type: WsEventType
  payload?: {
    dialog_id?: string
    message?: Message
    user_id?: string
    [key: string]: unknown
  }
}

/**
 * WebSocket message types to server
 */
export type WsClientMessageType = 'subscribe' | 'unsubscribe' | 'ping'

/**
 * WebSocket message to server
 */
export interface WsClientMessage {
  type: WsClientMessageType
  dialog_id?: string
}

// ============ SDK Configuration ============

/**
 * Supported UI locales
 */
export type Locale = 'ru' | 'en' | 'zh'

/**
 * User profile for display in chats
 * Contains default values that can be customized when joining specific dialogs
 */
export interface UserProfile {
  /** Display name (full name from user's profile) */
  displayName: string
  /** Company/organization name */
  company: string
  /** Contact email (optional) */
  email?: string
  /** Contact phone (optional) */
  phone?: string
}

/**
 * MTChat SDK configuration
 */
export interface MTChatConfig {
  /** API base URL */
  baseUrl: string
  /** WebSocket URL (derived from baseUrl if not provided) */
  wsUrl?: string
  /** Current user ID */
  userId: string
  /** User's scope configuration for access control */
  scopeConfig: ScopeConfig
  /** User's profile for display in chats */
  userProfile: UserProfile
  /** Callback when WebSocket connects */
  onConnect?: () => void
  /** Callback when WebSocket disconnects */
  onDisconnect?: () => void
  /** Callback on error */
  onError?: (error: Error) => void
  /** Callback on WebSocket message */
  onMessage?: (event: WsEvent) => void
  /** Enable auto-reconnect (default: true) */
  reconnect?: boolean
  /** Reconnect interval in ms (default: 3000) */
  reconnectInterval?: number
  /** Heartbeat interval in ms (default: 30000) */
  heartbeatInterval?: number
  /** UI locale (default: 'ru') */
  locale?: Locale
}

// ============ Component Props ============

/**
 * MTChat component display mode
 */
export type ChatMode = 'full' | 'inline'

/**
 * MTChat component props
 */
export interface MTChatProps {
  /** SDK configuration */
  config: MTChatConfig
  /** Display mode: 'full' (with dialog list) or 'inline' (single chat) */
  mode?: ChatMode
  /** For inline mode: object ID to show chat for */
  objectId?: string
  /** For inline mode: object type */
  objectType?: string
  /** Initial dialog ID (for full mode) */
  dialogId?: string
  /** Show header with dialog info */
  showHeader?: boolean
  /** Show sidebar with dialog list (full mode only) */
  showSidebar?: boolean
  /** Theme: 'light', 'dark', or 'custom' (uses inherited CSS variables) */
  theme?: 'light' | 'dark' | 'custom'
}

// ============ Composable Types ============

// ============ Virtual Scroll Types ============

/**
 * Virtual scroll item types for flat list rendering
 */
export type VirtualItemType = 'message' | 'date-divider' | 'unread-divider'

/**
 * Virtual scroll item wrapping messages and dividers
 * Used by DynamicScroller for efficient rendering
 */
export interface VirtualItem {
  /** Unique item ID (message ID or generated divider ID) */
  id: string
  /** Item type: message, date-divider, or unread-divider */
  type: VirtualItemType
  /** Message data (only for type='message') */
  message?: Message
  /** Formatted date string (only for type='date-divider') */
  date?: string
  /** Raw date for sorting/grouping (only for type='date-divider') */
  dateKey?: string
}

// ============ Composable Types ============

/**
 * useChat composable options
 */
export interface UseChatOptions {
  /** SDK configuration */
  config: MTChatConfig
  /** Auto-connect on mount (default: true) */
  autoConnect?: boolean
  /** Initial dialog ID to load */
  dialogId?: string
  /** For inline mode: object ID */
  objectId?: string
  /** For inline mode: object type */
  objectType?: string
}

/**
 * useChat composable return type
 */
export interface UseChatReturn {
  // State
  messages: import('vue').Ref<Message[]>
  participatingDialogs: import('vue').Ref<DialogListItem[]>
  availableDialogs: import('vue').Ref<DialogListItem[]>
  /** Archived dialogs for current user */
  archivedDialogs: import('vue').Ref<DialogListItem[]>
  participants: import('vue').Ref<DialogParticipant[]>
  currentDialog: import('vue').Ref<DialogListItem | null>
  isConnected: import('vue').Ref<boolean>
  isLoading: import('vue').Ref<boolean>
  error: import('vue').Ref<Error | null>
  /** ID of the first unread message (for divider) */
  firstUnreadMessageId: import('vue').Ref<string | null>
  /** Message being replied to */
  replyToMessage: import('vue').Ref<Message | null>
  /** Message being edited */
  editingMessage: import('vue').Ref<Message | null>
  /** Current search query for filtering dialogs */
  searchQuery: import('vue').Ref<string>
  /** Set of online user IDs */
  onlineUsers: import('vue').Ref<Set<string>>

  /** Cache for reply-to messages not in current page (null = deleted) */
  replyMessagesCache: import('vue').Ref<Map<string, Message | null>>
  /** Whether there are more older messages to load (scroll up) */
  hasMoreMessages: import('vue').Ref<boolean>
  /** Whether there are more newer messages to load (scroll down after jump) */
  hasMoreAfter: import('vue').Ref<boolean>
  /** Whether older messages are currently being loaded */
  isLoadingOlder: import('vue').Ref<boolean>
  /** Whether newer messages are currently being loaded */
  isLoadingNewer: import('vue').Ref<boolean>
  /** Whether we're jumping to a specific message */
  isJumpingToMessage: import('vue').Ref<boolean>
  /** Cooldown after jump to prevent scroll cascade */
  jumpCooldown: import('vue').Ref<boolean>

  // API access for file uploads
  api: import('../sdk/api').MTChatApi

  // Methods
  connect: () => void
  disconnect: () => void
  /** Set search query and reload dialogs */
  setSearchQuery: (query: string) => void
  sendMessage: (content: string, attachments?: AttachmentInput[]) => Promise<Message | undefined>
  /** Edit a message */
  editMessage: (messageId: string, content: string) => Promise<Message | undefined>
  /** Delete a message */
  deleteMessage: (messageId: string) => Promise<void>
  loadMessages: (options?: PaginationOptions) => Promise<void>
  /** Load older messages for infinite scroll (up) */
  loadOlderMessages: () => Promise<void>
  /** Load newer messages for infinite scroll (down after jump) */
  loadNewerMessages: () => Promise<void>
  /** Reset to latest messages (scroll-to-bottom after jump) */
  resetToLatest: () => Promise<void>
  /** Jump to a specific message (loads messages around it). Returns true if found, false if deleted. */
  jumpToMessage: (messageId: string) => Promise<boolean>
  /** Enable scroll cooldown to prevent scroll-triggered loading */
  enableScrollCooldown: () => void
  /** Get a reply-to message (from loaded or cache). Returns undefined if loading. */
  getReplyMessage: (messageId: string) => Message | null | undefined
  /** Fetch a single message for reply display */
  fetchReplyMessage: (messageId: string) => Promise<Message | null>
  loadParticipatingDialogs: () => Promise<void>
  /** Load archived dialogs (lazy) */
  loadArchivedDialogs: () => Promise<void>
  loadAvailableDialogs: () => Promise<void>
  loadDialogByObject: (objectType: string, objectId: string) => Promise<DialogListItem | null>
  selectDialog: (dialogId: string) => Promise<void>
  joinDialog: (dialogId: string, profile: JoinDialogRequest) => Promise<void>
  leaveDialog: (dialogId: string) => Promise<void>
  /** Archive a dialog for current user */
  archiveDialog: (dialogId: string) => Promise<void>
  /** Unarchive a dialog for current user */
  unarchiveDialog: (dialogId: string) => Promise<void>
  /** Pin a dialog for current user */
  pinDialog: (dialogId: string) => Promise<void>
  /** Unpin a dialog for current user */
  unpinDialog: (dialogId: string) => Promise<void>
  /** Toggle notifications for a dialog */
  toggleNotifications: (dialogId: string) => Promise<void>
  subscribe: (dialogId: string) => void
  unsubscribe: (dialogId: string) => void
  /** Mark messages as read up to specified message */
  markAsRead: (messageId?: string) => Promise<void>
  /** Set message to reply to */
  setReplyTo: (message: Message) => void
  /** Clear reply */
  clearReplyTo: () => void
  /** Set message to edit */
  setEditMessage: (message: Message) => void
  /** Clear edit mode */
  clearEditMessage: () => void
  /** Check if a user is currently online */
  isUserOnline: (userId: string) => boolean
}
