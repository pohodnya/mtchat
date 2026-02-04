/**
 * MTChat Vue Composable
 *
 * Reactive chat state management for Vue 3
 */

import { ref, onMounted, onUnmounted, type Ref } from 'vue'
import { MTChatClient } from '../sdk/client'
import type {
  Message,
  DialogListItem,
  DialogParticipant,
  PaginationOptions,
  UseChatOptions,
  UseChatReturn,
  AttachmentInput,
} from '../types'

/**
 * Vue composable for chat functionality
 */
export function useChat(options: UseChatOptions): UseChatReturn {
  const { config, autoConnect = true, dialogId, objectId, objectType } = options

  // Initialize client
  const client = new MTChatClient(config)

  // Reactive state
  const messages: Ref<Message[]> = ref([])
  const participatingDialogs: Ref<DialogListItem[]> = ref([])
  const availableDialogs: Ref<DialogListItem[]> = ref([])
  const participants: Ref<DialogParticipant[]> = ref([])
  const currentDialog: Ref<DialogListItem | null> = ref(null)
  const isConnected: Ref<boolean> = ref(false)
  const isLoading: Ref<boolean> = ref(false)
  const error: Ref<Error | null> = ref(null)
  const firstUnreadMessageId: Ref<string | null> = ref(null)
  const replyToMessage: Ref<Message | null> = ref(null)

  // Track subscribed dialog
  let subscribedDialogId: string | null = null

  // ============ Connection ============

  function connect(): void {
    client.connect()
  }

  function disconnect(): void {
    client.disconnect()
  }

  // ============ Dialogs ============

  async function loadParticipatingDialogs(): Promise<void> {
    try {
      isLoading.value = true
      error.value = null
      participatingDialogs.value = await client.api.getParticipatingDialogs()
    } catch (e) {
      error.value = e instanceof Error ? e : new Error(String(e))
      throw e
    } finally {
      isLoading.value = false
    }
  }

  async function loadAvailableDialogs(): Promise<void> {
    try {
      isLoading.value = true
      error.value = null
      availableDialogs.value = await client.api.getAvailableDialogs()
    } catch (e) {
      error.value = e instanceof Error ? e : new Error(String(e))
      throw e
    } finally {
      isLoading.value = false
    }
  }

  async function loadDialogByObject(
    objType: string,
    objId: string
  ): Promise<DialogListItem | null> {
    try {
      isLoading.value = true
      error.value = null
      const dialog = await client.api.getDialogByObject(objType, objId)
      if (dialog) {
        currentDialog.value = dialog
      }
      return dialog
    } catch (e) {
      error.value = e instanceof Error ? e : new Error(String(e))
      throw e
    } finally {
      isLoading.value = false
    }
  }

  async function selectDialog(id: string): Promise<void> {
    // Unsubscribe from previous dialog
    if (subscribedDialogId && subscribedDialogId !== id) {
      client.unsubscribe(subscribedDialogId)
    }

    // Find dialog in our lists
    let dialog = participatingDialogs.value.find((d) => d.id === id)
    if (!dialog) {
      dialog = availableDialogs.value.find((d) => d.id === id)
    }

    // If not found, fetch it
    if (!dialog) {
      try {
        const fetched = await client.api.getDialog(id)
        dialog = {
          ...fetched,
          participants_count: 0,
          i_am_participant: true,
        }
      } catch (e) {
        error.value = e instanceof Error ? e : new Error(String(e))
        throw e
      }
    }

    currentDialog.value = dialog
    subscribedDialogId = id

    // Subscribe and load messages
    client.subscribe(id)
    await loadMessages()
    await loadParticipants()
  }

  // ============ Join/Leave ============

  async function joinDialog(id: string): Promise<void> {
    try {
      isLoading.value = true
      error.value = null
      await client.api.joinDialog(id)

      // Move dialog from available to participating
      const dialogIndex = availableDialogs.value.findIndex((d) => d.id === id)
      if (dialogIndex !== -1) {
        const dialog = availableDialogs.value[dialogIndex]
        availableDialogs.value.splice(dialogIndex, 1)
        dialog.i_am_participant = true
        dialog.can_join = false
        participatingDialogs.value.push(dialog)
      }

      // Update current dialog if it's the one we joined
      if (currentDialog.value?.id === id) {
        currentDialog.value.i_am_participant = true
        currentDialog.value.can_join = false
      }
    } catch (e) {
      error.value = e instanceof Error ? e : new Error(String(e))
      throw e
    } finally {
      isLoading.value = false
    }
  }

  async function leaveDialog(id: string): Promise<void> {
    try {
      isLoading.value = true
      error.value = null
      await client.api.leaveDialog(id)

      // Remove from participating
      const dialogIndex = participatingDialogs.value.findIndex((d) => d.id === id)
      if (dialogIndex !== -1) {
        participatingDialogs.value.splice(dialogIndex, 1)
      }

      // If we left the current dialog, clear it
      if (currentDialog.value?.id === id) {
        client.unsubscribe(id)
        subscribedDialogId = null
        currentDialog.value = null
        messages.value = []
        participants.value = []
      }
    } catch (e) {
      error.value = e instanceof Error ? e : new Error(String(e))
      throw e
    } finally {
      isLoading.value = false
    }
  }

  // ============ Messages ============

  async function loadMessages(opts?: PaginationOptions): Promise<void> {
    if (!currentDialog.value) return

    try {
      isLoading.value = true
      error.value = null
      const response = await client.api.getMessages(currentDialog.value.id, opts)

      if (opts?.before) {
        // Prepend older messages
        messages.value = [...response.messages, ...messages.value]
      } else {
        messages.value = response.messages
        // Set first unread message ID for divider
        firstUnreadMessageId.value = response.first_unread_message_id || null
      }
    } catch (e) {
      error.value = e instanceof Error ? e : new Error(String(e))
      throw e
    } finally {
      isLoading.value = false
    }
  }

  async function markAsRead(messageId?: string): Promise<void> {
    if (!currentDialog.value) return

    // Use provided messageId or last message
    const lastReadId = messageId || messages.value[messages.value.length - 1]?.id
    if (!lastReadId) return

    try {
      await client.api.markAsRead(currentDialog.value.id, lastReadId)

      // Update unread count in dialog list (but keep divider until next visit)
      const dialog = participatingDialogs.value.find((d) => d.id === currentDialog.value?.id)
      if (dialog) {
        dialog.unread_count = 0
      }
      if (currentDialog.value) {
        currentDialog.value.unread_count = 0
      }
      // NOTE: firstUnreadMessageId is NOT cleared here
      // Divider stays visible until user leaves and re-enters the chat
    } catch (e) {
      // Non-critical, just log
      console.warn('Failed to mark as read:', e)
    }
  }

  async function loadParticipants(): Promise<void> {
    if (!currentDialog.value) return

    try {
      participants.value = await client.api.getParticipants(currentDialog.value.id)
    } catch (e) {
      // Non-critical, don't set error
      console.warn('Failed to load participants:', e)
    }
  }

  // ============ Reply ============

  function setReplyTo(message: Message): void {
    replyToMessage.value = message
  }

  function clearReplyTo(): void {
    replyToMessage.value = null
  }

  async function sendMessage(
    content: string,
    attachments?: AttachmentInput[]
  ): Promise<Message | undefined> {
    if (!currentDialog.value) return undefined

    try {
      error.value = null
      const message = await client.api.sendMessage(
        currentDialog.value.id,
        content,
        {
          replyTo: replyToMessage.value?.id,
          attachments,
        }
      )

      // Clear reply after sending
      clearReplyTo()

      // Add or update message (WebSocket may have added it without attachments)
      const existingIndex = messages.value.findIndex((m) => m.id === message.id)
      if (existingIndex !== -1) {
        // Update existing message with full data (including attachments)
        messages.value = [
          ...messages.value.slice(0, existingIndex),
          message,
          ...messages.value.slice(existingIndex + 1),
        ]
      } else {
        messages.value = [...messages.value, message]
      }

      // Sending a message marks all previous as read - clear divider
      firstUnreadMessageId.value = null

      return message
    } catch (e) {
      error.value = e instanceof Error ? e : new Error(String(e))
      throw e
    }
  }

  // ============ Subscriptions ============

  function subscribe(id: string): void {
    client.subscribe(id)
    subscribedDialogId = id
  }

  function unsubscribe(id: string): void {
    client.unsubscribe(id)
    if (subscribedDialogId === id) {
      subscribedDialogId = null
    }
  }

  // ============ Event Handlers ============

  function handleMessageNew(event: { payload?: { message?: Message }; id?: string; dialog_id?: string; sender_id?: string; content?: string; sent_at?: string }): void {
    // Support both formats:
    // 1. { payload: { message: {...} } } - expected format
    // 2. { id, dialog_id, sender_id, content, sent_at } - backend format
    let message: Message | undefined = event.payload?.message

    if (!message && event.id && event.dialog_id) {
      // Backend sends flat structure
      message = {
        id: event.id,
        dialog_id: event.dialog_id,
        sender_id: event.sender_id!,
        content: event.content!,
        sent_at: event.sent_at!,
      }
    }

    if (!message) return

    // Only add if it's for the current dialog
    if (currentDialog.value && message.dialog_id === currentDialog.value.id) {
      // Check for duplicates
      if (!messages.value.some((m) => m.id === message!.id)) {
        messages.value = [...messages.value, message]
      }
    }
  }

  function handleParticipantJoined(event: { payload?: { dialog_id?: string; user_id?: string } }): void {
    const { dialog_id, user_id } = event.payload || {}
    if (!dialog_id || !user_id) return

    // Update participant count in dialogs
    const updateCount = (dialogs: DialogListItem[]) => {
      const dialog = dialogs.find((d) => d.id === dialog_id)
      if (dialog) {
        dialog.participants_count = (dialog.participants_count || 0) + 1
      }
    }

    updateCount(participatingDialogs.value)
    updateCount(availableDialogs.value)

    if (currentDialog.value?.id === dialog_id) {
      currentDialog.value.participants_count = (currentDialog.value.participants_count || 0) + 1
      // Reload participants list
      loadParticipants()
    }
  }

  function handleParticipantLeft(event: { payload?: { dialog_id?: string; user_id?: string } }): void {
    const { dialog_id, user_id } = event.payload || {}
    if (!dialog_id || !user_id) return

    // Update participant count in dialogs
    const updateCount = (dialogs: DialogListItem[]) => {
      const dialog = dialogs.find((d) => d.id === dialog_id)
      if (dialog && dialog.participants_count > 0) {
        dialog.participants_count--
      }
    }

    updateCount(participatingDialogs.value)
    updateCount(availableDialogs.value)

    if (currentDialog.value?.id === dialog_id) {
      if (currentDialog.value.participants_count > 0) {
        currentDialog.value.participants_count--
      }
      // Reload participants list
      loadParticipants()
    }
  }

  function handleMessageRead(event: { dialog_id?: string; user_id?: string; payload?: { dialog_id?: string; user_id?: string } }): void {
    // Support both flat and payload formats
    const dialog_id = event.dialog_id || event.payload?.dialog_id
    const readByUserId = event.user_id || event.payload?.user_id
    if (!dialog_id || !readByUserId) return

    // Only handle if it's the current user's read receipt
    if (readByUserId === config.userId) {
      // Update unread count in dialog list
      const dialog = participatingDialogs.value.find((d) => d.id === dialog_id)
      if (dialog) {
        dialog.unread_count = 0
      }
      // Update current dialog unread count (but keep divider)
      if (currentDialog.value?.id === dialog_id) {
        currentDialog.value.unread_count = 0
        // NOTE: firstUnreadMessageId is NOT cleared
        // Divider stays visible until user re-enters the chat
      }
    }
  }

  // ============ Lifecycle ============

  onMounted(async () => {
    // Set up event handlers
    client.on('connected', () => {
      isConnected.value = true
    })

    client.on('disconnected' as any, () => {
      isConnected.value = false
    })

    client.on('message.new', handleMessageNew as any)
    client.on('message.read', handleMessageRead as any)
    client.on('participant.joined', handleParticipantJoined as any)
    client.on('participant.left', handleParticipantLeft as any)

    // Auto-connect
    if (autoConnect) {
      connect()
    }

    // Load initial dialog
    if (objectType && objectId) {
      // Inline mode - load by object
      await loadDialogByObject(objectType, objectId)
      if (currentDialog.value) {
        subscribe(currentDialog.value.id)
        await loadMessages()
      }
    } else if (dialogId) {
      // Full mode with initial dialog
      await selectDialog(dialogId)
    }
  })

  onUnmounted(() => {
    // Cleanup
    if (subscribedDialogId) {
      client.unsubscribe(subscribedDialogId)
    }
    client.disconnect()
  })

  return {
    // State
    messages,
    participatingDialogs,
    availableDialogs,
    participants,
    currentDialog,
    isConnected,
    isLoading,
    error,
    firstUnreadMessageId,
    replyToMessage,

    // API access for file uploads
    api: client.api,

    // Methods
    connect,
    disconnect,
    sendMessage,
    loadMessages,
    loadParticipatingDialogs,
    loadAvailableDialogs,
    loadDialogByObject,
    selectDialog,
    joinDialog,
    leaveDialog,
    subscribe,
    unsubscribe,
    markAsRead,
    setReplyTo,
    clearReplyTo,
  }
}

export type { UseChatOptions, UseChatReturn }
