/**
 * MTChat Vue SDK
 *
 * v3 Architecture: Object-bound dialogs with scope-based access
 *
 * @example Full mode (with dialog list)
 * ```vue
 * <MTChat :config="config" mode="full" />
 * ```
 *
 * @example Inline mode (single chat for object)
 * ```vue
 * <MTChat
 *   :config="config"
 *   mode="inline"
 *   object-type="tender"
 *   :object-id="tenderId"
 * />
 * ```
 *
 * @example Composable usage
 * ```ts
 * const { messages, sendMessage, joinDialog } = useChat({ config })
 * ```
 */

// Vue Components
export { default as MTChat } from './components/MTChat.vue'
export { default as FileViewer } from './components/chat/FileViewer.vue'
export { default as Icon } from './components/Icon.vue'
export type { IconName } from './components/Icon.vue'

// SDK Classes
export { MTChatClient, MTChatApi, MTChatWebSocket } from './sdk'
export type { WsEventHandler } from './sdk'

// Composables
export { useChat } from './composables/useChat'
export type { UseChatOptions, UseChatReturn } from './composables/useChat'
export { useFileUpload } from './composables/useFileUpload'
export type { UseFileUploadOptions, UseFileUploadReturn } from './composables/useFileUpload'

// Types
export type {
  // Core entities
  Dialog,
  DialogListItem,
  DialogParticipant,
  DialogAccessScope,
  Message,

  // Attachments
  Attachment,
  PendingAttachment,
  AttachmentInput,
  PresignUploadResponse,
  AttachmentType,

  // Configuration
  MTChatConfig,
  ScopeConfig,
  ChatMode,
  MTChatProps,
  Locale,

  // API types
  ApiResponse,
  PaginationOptions,
  DialogListType,

  // WebSocket types
  WsEvent,
  WsEventType,
  WsClientMessage,
  WsClientMessageType,
} from './types'

// Attachment utilities
export {
  getAttachmentType,
  isAllowedFileType,
  isValidFileSize,
  formatFileSize,
  ATTACHMENT_LIMITS,
} from './types'
