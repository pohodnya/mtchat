/**
 * MTChat Vue PrimeVue Integration
 *
 * Complete MTChat SDK with PrimeVue components and theme integration.
 * Uses your PrimeVue preset tokens (--p-*) for consistent styling.
 *
 * All types, composables and utilities are re-exported from @mtchat/vue,
 * so you only need this single package.
 *
 * @example Basic usage
 * ```vue
 * <script setup>
 * import { MTChatPrime, type MTChatConfig, type Message } from '@mtchat/vue-primevue'
 *
 * const config: MTChatConfig = {
 *   baseUrl: 'https://chat.example.com',
 *   token: userToken,
 *   userId: user.id,
 * }
 * </script>
 * <template>
 *   <MTChatPrime :config="config" />
 * </template>
 * ```
 *
 * @example Dark theme
 * ```vue
 * <MTChatPrime :config="config" theme="dark" />
 * ```
 *
 * @example Using composables
 * ```ts
 * import { useChat, type UseChatOptions } from '@mtchat/vue-primevue'
 *
 * const { messages, sendMessage } = useChat({ config })
 * ```
 *
 * Theme customization:
 * - Override PrimeVue tokens (--p-*) via your preset configuration
 * - Override MTChat tokens (--mtchat-*) via CSS on .mtchat-prime class
 */

// Ready-to-use component with PrimeVue registry pre-configured
export { default as MTChatPrime } from './components/MTChatPrime.vue'

// Registry
export { primevueRegistry } from './registry/primevueRegistry'

// Individual PrimeVue adapter components (for custom registry building)
export {
  PrimeButton,
  PrimeDialog,
  PrimeMenu,
  PrimeContextMenu,
  PrimeInput,
  PrimeCheckbox,
  PrimeRadioButton,
  PrimeTabs,
  PrimeTab,
  PrimeAccordion,
  PrimeAccordionPanel,
} from './primitives'

// ============================================================
// Re-exports from @mtchat/vue for single-package usage
// ============================================================

// Core types
export type {
  // Entities
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
  WsEventHandler,

  // Composable types
  UseChatOptions,
  UseChatReturn,
  UseFileUploadOptions,
  UseFileUploadReturn,

  // Registry types
  ComponentRegistry,
  PartialRegistry,
  MtButtonProps,
  MtDialogProps,
  MtMenuProps,
  MtMenuItem,
  MtInputProps,
  MtCheckboxProps,
  MtRadioButtonProps,
  MtTabsProps,
  MtTabProps,
  MtAccordionProps,
  MtAccordionPanelProps,
} from '@mtchat/vue'

// Composables
export { useChat, useFileUpload } from '@mtchat/vue'

// SDK classes (for advanced usage)
export { MTChatClient, MTChatApi, MTChatWebSocket } from '@mtchat/vue'

// Attachment utilities
export {
  getAttachmentType,
  isAllowedFileType,
  isValidFileSize,
  formatFileSize,
  ATTACHMENT_LIMITS,
} from '@mtchat/vue'

// Base MTChat component (for custom registry scenarios)
export { MTChat, FileViewer, Icon, provideRegistry, useRegistry } from '@mtchat/vue'
