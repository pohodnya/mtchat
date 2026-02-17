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
 * @example With PrimeVue components
 * ```vue
 * <script setup>
 * import { MTChat, provideRegistry } from '@mtchat/vue'
 * import { primevueRegistry } from '@mtchat/vue-primevue'
 * provideRegistry(primevueRegistry)
 * </script>
 * <template>
 *   <MTChat :config="config" />
 * </template>
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

// Component Registry (for swapping UI primitives)
export { provideRegistry, useRegistry, REGISTRY_KEY } from './registry'
export type { ComponentRegistry, PartialRegistry } from './registry'

// Native primitives (for custom registry building)
export {
  MtButton,
  MtDialog,
  MtMenu,
  MtContextMenu,
  MtInput,
  MtCheckbox,
  MtRadioButton,
  MtTabs,
  MtTab,
  MtAccordion,
  MtAccordionPanel,
  vTooltip,
} from './primitives'

// Primitive prop types (for custom implementations)
export type {
  MtButtonProps,
  MtButtonVariant,
  MtButtonSize,
  MtDialogProps,
  MtDialogEmits,
  MtMenuProps,
  MtMenuEmits,
  MtMenuExpose,
  MtMenuItem,
  MtContextMenuProps,
  MtContextMenuEmits,
  MtContextMenuExpose,
  MtInputProps,
  MtInputEmits,
  MtInputExpose,
  MtCheckboxProps,
  MtCheckboxEmits,
  MtRadioButtonProps,
  MtRadioButtonEmits,
  MtTabsProps,
  MtTabsEmits,
  MtTabProps,
  MtAccordionProps,
  MtAccordionEmits,
  MtAccordionPanelProps,
  MtTooltipOptions,
} from './registry'

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
  WsEventPayload,
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
