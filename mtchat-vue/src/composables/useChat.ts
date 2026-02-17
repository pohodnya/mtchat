/**
 * MTChat Vue Composable
 *
 * Reactive chat state management for Vue 3
 */

import { ref, computed, onMounted, onUnmounted, type Ref, type ComputedRef } from 'vue'
import { MTChatClient } from '../sdk/client'
import type {
  Message,
  DialogListItem,
  DialogParticipant,
  PaginationOptions,
  UseChatOptions,
  UseChatReturn,
  AttachmentInput,
  JoinDialogRequest,
  WsEvent,
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
  const archivedDialogs: Ref<DialogListItem[]> = ref([])
  const participants: Ref<DialogParticipant[]> = ref([])
  const currentDialog: Ref<DialogListItem | null> = ref(null)
  const isConnected: Ref<boolean> = ref(false)
  // Split loading states to prevent race conditions between concurrent operations
  const isLoadingDialogs: Ref<boolean> = ref(false)
  const isLoadingMessages: Ref<boolean> = ref(false)
  const isActionLoading: Ref<boolean> = ref(false)
  // Combined loading state for backward compatibility (used by UI to disable buttons)
  const isLoading: ComputedRef<boolean> = computed(() =>
    isLoadingDialogs.value || isLoadingMessages.value || isActionLoading.value
  )
  const error: Ref<Error | null> = ref(null)
  const firstUnreadMessageId: Ref<string | null> = ref(null)
  const replyToMessage: Ref<Message | null> = ref(null)
  const editingMessage: Ref<Message | null> = ref(null)
  const searchQuery: Ref<string> = ref('')
  const onlineUsers: Ref<Set<string>> = ref(new Set())

  // Reply message cache (for messages not in current page due to pagination)
  // Map<messageId, Message | null> - null means message was deleted/not found
  const replyMessagesCache: Ref<Map<string, Message | null>> = ref(new Map())
  const pendingReplyFetches = new Set<string>()

  // Pagination state for infinite scroll (bidirectional)
  const hasMoreMessages: Ref<boolean> = ref(true)  // more messages before (older)
  const hasMoreAfter: Ref<boolean> = ref(false)    // more messages after (newer)
  const isLoadingOlder: Ref<boolean> = ref(false)
  const isLoadingNewer: Ref<boolean> = ref(false)
  const oldestMessageId: Ref<string | null> = ref(null)
  const newestMessageId: Ref<string | null> = ref(null)

  // Jump to message state
  const isJumpingToMessage: Ref<boolean> = ref(false)
  // Cooldown after jump to prevent scroll cascade
  const jumpCooldown: Ref<boolean> = ref(false)

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
      isLoadingDialogs.value = true
      error.value = null
      const search = searchQuery.value || undefined

      // Load only active dialogs (non-archived)
      // Archived dialogs are loaded lazily when accordion is opened
      const active = await client.api.getParticipatingDialogs(search, false)
      participatingDialogs.value = active
    } catch (e) {
      error.value = e instanceof Error ? e : new Error(String(e))
      throw e
    } finally {
      isLoadingDialogs.value = false
    }
  }

  async function loadArchivedDialogs(): Promise<void> {
    try {
      isLoadingDialogs.value = true
      error.value = null
      const search = searchQuery.value || undefined

      const archived = await client.api.getParticipatingDialogs(search, true)
      archivedDialogs.value = archived
    } catch (e) {
      error.value = e instanceof Error ? e : new Error(String(e))
      throw e
    } finally {
      isLoadingDialogs.value = false
    }
  }

  async function loadAvailableDialogs(): Promise<void> {
    try {
      isLoadingDialogs.value = true
      error.value = null
      const search = searchQuery.value || undefined
      availableDialogs.value = await client.api.getAvailableDialogs(search)
    } catch (e) {
      error.value = e instanceof Error ? e : new Error(String(e))
      throw e
    } finally {
      isLoadingDialogs.value = false
    }
  }

  /**
   * Set search query for dialog filtering
   */
  function setSearchQuery(query: string): void {
    searchQuery.value = query
  }

  async function loadDialogByObject(
    objType: string,
    objId: string
  ): Promise<DialogListItem | null> {
    try {
      isLoadingDialogs.value = true
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
      isLoadingDialogs.value = false
    }
  }

  async function selectDialog(id: string): Promise<void> {
    // Unsubscribe from previous dialog
    if (subscribedDialogId && subscribedDialogId !== id) {
      client.unsubscribe(subscribedDialogId)
    }

    // Clear messages and cache from previous dialog
    messages.value = []
    firstUnreadMessageId.value = null
    replyMessagesCache.value = new Map()
    pendingReplyFetches.clear()
    hasMoreMessages.value = true
    isLoadingOlder.value = false
    oldestMessageId.value = null

    // Find dialog in our lists (active, archived, or available)
    let dialog = participatingDialogs.value.find((d) => d.id === id)
    if (!dialog) {
      dialog = archivedDialogs.value.find((d) => d.id === id)
    }
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

  async function joinDialog(id: string, profile: JoinDialogRequest): Promise<void> {
    try {
      isActionLoading.value = true
      error.value = null
      await client.api.joinDialog(id, profile)

      // Move dialog from available to participating (immutable updates)
      const dialogIndex = availableDialogs.value.findIndex((d) => d.id === id)
      if (dialogIndex !== -1) {
        const dialog = { ...availableDialogs.value[dialogIndex], i_am_participant: true, can_join: false }
        availableDialogs.value = availableDialogs.value.filter((d) => d.id !== id)
        participatingDialogs.value = [...participatingDialogs.value, dialog]
      } else {
        // Dialog wasn't in available list - reload both lists
        await loadParticipatingDialogs()
        await loadAvailableDialogs()
      }

      // Update current dialog if it's the one we joined
      if (currentDialog.value?.id === id) {
        currentDialog.value = {
          ...currentDialog.value,
          i_am_participant: true,
          can_join: false,
        }

        // Now that we're a participant, load messages and participants
        await loadMessages()
        await loadParticipants()
      }
    } catch (e) {
      error.value = e instanceof Error ? e : new Error(String(e))
      throw e
    } finally {
      isActionLoading.value = false
    }
  }

  async function leaveDialog(id: string): Promise<void> {
    try {
      isActionLoading.value = true
      error.value = null
      await client.api.leaveDialog(id)

      // Remove from participating (immutable update)
      participatingDialogs.value = participatingDialogs.value.filter((d) => d.id !== id)

      // Also remove from archived if present
      archivedDialogs.value = archivedDialogs.value.filter((d) => d.id !== id)

      // Reload available dialogs - the left dialog may now be available again
      await loadAvailableDialogs()

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
      isActionLoading.value = false
    }
  }

  // ============ Archive ============

  async function archiveDialog(dialogId: string): Promise<void> {
    try {
      isActionLoading.value = true
      error.value = null
      await client.api.archiveDialog(dialogId)

      // Move from active to archived (immutable updates)
      const idx = participatingDialogs.value.findIndex((d) => d.id === dialogId)
      if (idx !== -1) {
        const dialog = { ...participatingDialogs.value[idx], is_archived: true }
        participatingDialogs.value = participatingDialogs.value.filter((d) => d.id !== dialogId)
        archivedDialogs.value = [...archivedDialogs.value, dialog]
      }

      // Clear current dialog if it was archived
      if (currentDialog.value?.id === dialogId) {
        currentDialog.value = null
        messages.value = []
      }
    } catch (e) {
      error.value = e instanceof Error ? e : new Error(String(e))
      throw e
    } finally {
      isActionLoading.value = false
    }
  }

  async function unarchiveDialog(dialogId: string): Promise<void> {
    try {
      isActionLoading.value = true
      error.value = null
      await client.api.unarchiveDialog(dialogId)

      // Move from archived to active (immutable updates)
      const idx = archivedDialogs.value.findIndex((d) => d.id === dialogId)
      if (idx !== -1) {
        const dialog = { ...archivedDialogs.value[idx], is_archived: false }
        archivedDialogs.value = archivedDialogs.value.filter((d) => d.id !== dialogId)
        participatingDialogs.value = [...participatingDialogs.value, dialog]
      }

      // Update current dialog if it was unarchived
      if (currentDialog.value?.id === dialogId) {
        currentDialog.value = { ...currentDialog.value, is_archived: false }
      }
    } catch (e) {
      error.value = e instanceof Error ? e : new Error(String(e))
      throw e
    } finally {
      isActionLoading.value = false
    }
  }

  // ============ Pin/Unpin ============

  async function pinDialog(dialogId: string): Promise<void> {
    try {
      error.value = null
      await client.api.pinDialog(dialogId)

      // Update local state (immutable)
      const idx = participatingDialogs.value.findIndex((d) => d.id === dialogId)
      if (idx !== -1) {
        participatingDialogs.value = [
          ...participatingDialogs.value.slice(0, idx),
          { ...participatingDialogs.value[idx], is_pinned: true },
          ...participatingDialogs.value.slice(idx + 1),
        ]
      }

      // Update current dialog if it was pinned
      if (currentDialog.value?.id === dialogId) {
        currentDialog.value = { ...currentDialog.value, is_pinned: true }
      }
    } catch (e) {
      error.value = e instanceof Error ? e : new Error(String(e))
      throw e
    }
  }

  async function unpinDialog(dialogId: string): Promise<void> {
    try {
      error.value = null
      await client.api.unpinDialog(dialogId)

      // Update local state (immutable)
      const idx = participatingDialogs.value.findIndex((d) => d.id === dialogId)
      if (idx !== -1) {
        participatingDialogs.value = [
          ...participatingDialogs.value.slice(0, idx),
          { ...participatingDialogs.value[idx], is_pinned: false },
          ...participatingDialogs.value.slice(idx + 1),
        ]
      }

      // Update current dialog if it was unpinned
      if (currentDialog.value?.id === dialogId) {
        currentDialog.value = { ...currentDialog.value, is_pinned: false }
      }
    } catch (e) {
      error.value = e instanceof Error ? e : new Error(String(e))
      throw e
    }
  }

  // ============ Notifications ============

  async function toggleNotifications(dialogId: string): Promise<void> {
    try {
      error.value = null

      // Find current state in active or archived dialogs
      const dialog =
        participatingDialogs.value.find((d) => d.id === dialogId) ||
        archivedDialogs.value.find((d) => d.id === dialogId)

      const newEnabled = !(dialog?.notifications_enabled ?? true)

      await client.api.setDialogNotifications(dialogId, newEnabled)

      // Update local state in active dialogs (immutable)
      const activeIdx = participatingDialogs.value.findIndex((d) => d.id === dialogId)
      if (activeIdx !== -1) {
        participatingDialogs.value = [
          ...participatingDialogs.value.slice(0, activeIdx),
          { ...participatingDialogs.value[activeIdx], notifications_enabled: newEnabled },
          ...participatingDialogs.value.slice(activeIdx + 1),
        ]
      }

      // Update local state in archived dialogs (immutable)
      const archivedIdx = archivedDialogs.value.findIndex((d) => d.id === dialogId)
      if (archivedIdx !== -1) {
        archivedDialogs.value = [
          ...archivedDialogs.value.slice(0, archivedIdx),
          { ...archivedDialogs.value[archivedIdx], notifications_enabled: newEnabled },
          ...archivedDialogs.value.slice(archivedIdx + 1),
        ]
      }

      // Update current dialog if it's the one we toggled
      if (currentDialog.value?.id === dialogId) {
        currentDialog.value = {
          ...currentDialog.value,
          notifications_enabled: newEnabled,
        }
      }
    } catch (e) {
      error.value = e instanceof Error ? e : new Error(String(e))
      throw e
    }
  }

  // ============ Messages ============

  async function loadMessages(opts?: PaginationOptions): Promise<void> {
    if (!currentDialog.value) return

    // Don't load messages if not a participant
    if (!currentDialog.value.i_am_participant) {
      messages.value = []
      firstUnreadMessageId.value = null
      return
    }

    try {
      isLoadingMessages.value = true
      error.value = null

      // Default limit for pagination
      const limit = opts?.limit ?? 50
      const response = await client.api.getMessages(currentDialog.value.id, { ...opts, limit })

      if (opts?.before) {
        // Prepend older messages
        messages.value = [...response.messages, ...messages.value]
      } else {
        // Set first unread message ID BEFORE messages so the watcher sees it
        firstUnreadMessageId.value = response.first_unread_message_id || null
        messages.value = response.messages
      }

      // Track pagination state
      hasMoreMessages.value = response.has_more_before ?? (response.messages.length >= limit)
      hasMoreAfter.value = false // Loading latest means no newer messages

      // Update oldest and newest message IDs for cursor-based pagination
      if (response.messages.length > 0) {
        const oldestInResponse = response.messages[0]
        const newestInResponse = response.messages[response.messages.length - 1]

        if (!oldestMessageId.value || new Date(oldestInResponse.sent_at) < new Date(oldestMessageId.value)) {
          oldestMessageId.value = oldestInResponse.id
        }
        newestMessageId.value = newestInResponse.id
      }
    } catch (e) {
      // 403 = not a participant, expected for potential participants
      const err = e instanceof Error ? e : new Error(String(e))
      if (err.message.includes('403') || err.message.includes('Forbidden')) {
        messages.value = []
        firstUnreadMessageId.value = null
        return
      }
      error.value = err
      throw e
    } finally {
      isLoadingMessages.value = false
    }
  }

  /**
   * Load older messages (for infinite scroll)
   */
  async function loadOlderMessages(): Promise<void> {
    if (!currentDialog.value || !hasMoreMessages.value || isLoadingOlder.value) return

    // Don't load if not a participant
    if (!currentDialog.value.i_am_participant) return

    // Need oldest message ID for cursor
    if (!oldestMessageId.value) return

    try {
      isLoadingOlder.value = true
      error.value = null

      const limit = 50
      const response = await client.api.getMessages(currentDialog.value.id, {
        before: oldestMessageId.value,
        limit,
      })

      // Prepend older messages
      if (response.messages.length > 0) {
        messages.value = [...response.messages, ...messages.value]

        // Update oldest message ID
        oldestMessageId.value = response.messages[0].id
      }

      // Check if there are more messages
      hasMoreMessages.value = response.messages.length >= limit
    } catch (e) {
      const err = e instanceof Error ? e : new Error(String(e))
      // Don't set error for pagination failures - just stop loading more
      console.warn('Failed to load older messages:', err)
      hasMoreMessages.value = false
    } finally {
      isLoadingOlder.value = false
      enableScrollCooldown()
    }
  }

  /**
   * Load newer messages (infinite scroll down after jumping to a message)
   */
  async function loadNewerMessages(): Promise<void> {
    if (!currentDialog.value || !hasMoreAfter.value || isLoadingNewer.value) return

    // Don't load if not a participant
    if (!currentDialog.value.i_am_participant) return

    // Need newest message ID for cursor
    if (!newestMessageId.value) return

    try {
      isLoadingNewer.value = true
      error.value = null

      const limit = 50
      const response = await client.api.getMessages(currentDialog.value.id, {
        after: newestMessageId.value,
        limit,
      })

      // Append newer messages
      if (response.messages.length > 0) {
        messages.value = [...messages.value, ...response.messages]

        // Update newest message ID
        newestMessageId.value = response.messages[response.messages.length - 1].id
      }

      // Check if there are more messages after
      hasMoreAfter.value = response.has_more_after ?? false
    } catch (e) {
      const err = e instanceof Error ? e : new Error(String(e))
      console.warn('Failed to load newer messages:', err)
      hasMoreAfter.value = false
    } finally {
      isLoadingNewer.value = false
      enableScrollCooldown()
    }
  }

  /**
   * Reset to latest messages (used by scroll-to-bottom button after jumping)
   * This reloads the latest messages and clears the "after" pagination state
   */
  async function resetToLatest(): Promise<void> {
    if (!currentDialog.value) return

    // Don't reset if not a participant
    if (!currentDialog.value.i_am_participant) return

    try {
      isLoadingMessages.value = true
      jumpCooldown.value = true
      error.value = null

      const limit = 50
      const response = await client.api.getMessages(currentDialog.value.id, { limit })

      // Replace messages with latest
      messages.value = response.messages
      firstUnreadMessageId.value = response.first_unread_message_id ?? null

      // Reset pagination state
      hasMoreMessages.value = response.has_more_before ?? true
      hasMoreAfter.value = false // We're at the latest - no more after

      if (response.messages.length > 0) {
        oldestMessageId.value = response.messages[0].id
        newestMessageId.value = response.messages[response.messages.length - 1].id
      }
    } catch (e) {
      error.value = e instanceof Error ? e : new Error(String(e))
      throw e
    } finally {
      isLoadingMessages.value = false
      enableScrollCooldown()
    }
  }

  /**
   * Get a reply-to message from loaded messages or cache
   * Returns: Message if found, null if deleted/not found, undefined if loading
   */
  function getReplyMessage(messageId: string): Message | null | undefined {
    // First check loaded messages
    const loaded = messages.value.find((m) => m.id === messageId)
    if (loaded) return loaded

    // Check cache
    if (replyMessagesCache.value.has(messageId)) {
      return replyMessagesCache.value.get(messageId)
    }

    // Not found and not in cache - return undefined (loading state)
    return undefined
  }

  /**
   * Fetch a single message for reply display
   * Returns the message or null if not found
   */
  async function fetchReplyMessage(messageId: string): Promise<Message | null> {
    // Already in cache
    if (replyMessagesCache.value.has(messageId)) {
      return replyMessagesCache.value.get(messageId) ?? null
    }

    // Already being fetched
    if (pendingReplyFetches.has(messageId)) {
      return null
    }

    // Check loaded messages first
    const loaded = messages.value.find((m) => m.id === messageId)
    if (loaded) {
      return loaded
    }

    if (!currentDialog.value) return null

    try {
      pendingReplyFetches.add(messageId)
      const message = await client.api.getMessage(currentDialog.value.id, messageId)

      // Update cache (create new Map for reactivity)
      const newCache = new Map(replyMessagesCache.value)
      newCache.set(messageId, message)
      replyMessagesCache.value = newCache

      return message
    } catch (e) {
      // Message not found (deleted or doesn't exist)
      const newCache = new Map(replyMessagesCache.value)
      newCache.set(messageId, null)
      replyMessagesCache.value = newCache
      return null
    } finally {
      pendingReplyFetches.delete(messageId)
    }
  }

  /**
   * Jump to a specific message by loading messages around it
   * Used when clicking on a quoted message that's not in the current loaded set
   * Returns true if the message was found and loaded, false if message is deleted
   */
  async function jumpToMessage(messageId: string): Promise<boolean> {
    // 1. Check if already loaded
    const exists = messages.value.some((m) => m.id === messageId)
    if (exists) {
      return true
    }

    // 2. Check if deleted (from cache)
    if (replyMessagesCache.value.get(messageId) === null) {
      return false
    }

    if (!currentDialog.value) {
      return false
    }

    // 3. Load messages around target
    isJumpingToMessage.value = true
    jumpCooldown.value = true
    try {
      const response = await client.api.getMessages(currentDialog.value.id, {
        around: messageId,
        limit: 50,
      })

      // 4. Replace messages with the new set
      messages.value = response.messages

      // 5. Update pagination state for bidirectional scroll
      hasMoreMessages.value = response.has_more_before ?? true
      hasMoreAfter.value = response.has_more_after ?? false

      if (response.messages.length > 0) {
        oldestMessageId.value = response.messages[0].id
        newestMessageId.value = response.messages[response.messages.length - 1].id
      }

      // Check if the target message was found
      return response.messages.some((m) => m.id === messageId)
    } catch (e) {
      console.warn('[useChat] Failed to jump to message:', e)
      return false
    } finally {
      isJumpingToMessage.value = false
      enableScrollCooldown()
    }
  }

  /**
   * Enable scroll cooldown (prevents scroll-triggered loading)
   * Used when programmatically scrolling (jump to message, scroll-to-bottom button)
   */
  function enableScrollCooldown(): void {
    jumpCooldown.value = true
    setTimeout(() => {
      jumpCooldown.value = false
    }, 300)
  }

  async function markAsRead(messageId?: string): Promise<void> {
    if (!currentDialog.value) return

    // Use provided messageId or last message
    const lastReadId = messageId || messages.value[messages.value.length - 1]?.id
    if (!lastReadId) return

    try {
      await client.api.markAsRead(currentDialog.value.id, lastReadId)

      const dialogId = currentDialog.value.id

      // Update unread count in active dialogs
      const activeIdx = participatingDialogs.value.findIndex((d) => d.id === dialogId)
      if (activeIdx !== -1) {
        participatingDialogs.value[activeIdx] = {
          ...participatingDialogs.value[activeIdx],
          unread_count: 0,
        }
      }

      // Update unread count in archived dialogs
      const archivedIdx = archivedDialogs.value.findIndex((d) => d.id === dialogId)
      if (archivedIdx !== -1) {
        archivedDialogs.value[archivedIdx] = {
          ...archivedDialogs.value[archivedIdx],
          unread_count: 0,
        }
      }

      // Update current dialog
      if (currentDialog.value) {
        currentDialog.value = {
          ...currentDialog.value,
          unread_count: 0,
        }
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
      const loadedParticipants = await client.api.getParticipants(currentDialog.value.id)
      participants.value = loadedParticipants

      // Populate onlineUsers from is_online field
      const online = loadedParticipants.filter((p) => p.is_online).map((p) => p.user_id)
      onlineUsers.value = new Set(online)
    } catch (e) {
      // 403 = no access, clear participants
      const err = e instanceof Error ? e : new Error(String(e))
      if (err.message.includes('403') || err.message.includes('Forbidden')) {
        participants.value = []
        return
      }
      // Non-critical, don't set error
      console.warn('Failed to load participants:', e)
    }
  }

  // ============ Helper: Update dialog last_message_at ============

  function updateDialogLastMessageAt(dialogId: string, sentAt: string): void {
    // Update in participating dialogs
    const activeIdx = participatingDialogs.value.findIndex((d) => d.id === dialogId)
    if (activeIdx !== -1) {
      participatingDialogs.value[activeIdx] = {
        ...participatingDialogs.value[activeIdx],
        last_message_at: sentAt,
      }
    }

    // Update in archived dialogs
    const archivedIdx = archivedDialogs.value.findIndex((d) => d.id === dialogId)
    if (archivedIdx !== -1) {
      archivedDialogs.value[archivedIdx] = {
        ...archivedDialogs.value[archivedIdx],
        last_message_at: sentAt,
      }
    }

    // Update current dialog
    if (currentDialog.value?.id === dialogId) {
      currentDialog.value = {
        ...currentDialog.value,
        last_message_at: sentAt,
      }
    }
  }

  // ============ Reply ============

  function setReplyTo(message: Message): void {
    replyToMessage.value = message
    // Clear edit mode when setting reply
    editingMessage.value = null
  }

  function clearReplyTo(): void {
    replyToMessage.value = null
  }

  // ============ Edit/Delete ============

  function setEditMessage(message: Message): void {
    editingMessage.value = message
    // Clear reply when editing
    replyToMessage.value = null
  }

  function clearEditMessage(): void {
    editingMessage.value = null
  }

  async function editMessage(messageId: string, content: string): Promise<Message | undefined> {
    if (!currentDialog.value) return undefined

    try {
      error.value = null
      const updated = await client.api.editMessage(currentDialog.value.id, messageId, content)

      // Update in local list
      const idx = messages.value.findIndex((m) => m.id === messageId)
      if (idx !== -1) {
        messages.value = [
          ...messages.value.slice(0, idx),
          updated,
          ...messages.value.slice(idx + 1),
        ]
      }

      clearEditMessage()
      return updated
    } catch (e) {
      error.value = e instanceof Error ? e : new Error(String(e))
      throw e
    }
  }

  async function deleteMessage(messageId: string): Promise<void> {
    if (!currentDialog.value) return

    try {
      error.value = null
      await client.api.deleteMessage(currentDialog.value.id, messageId)

      // Remove from local list
      messages.value = messages.value.filter((m) => m.id !== messageId)
    } catch (e) {
      error.value = e instanceof Error ? e : new Error(String(e))
      throw e
    }
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

      // Update last_message_at so dialog moves up in the list
      updateDialogLastMessageAt(currentDialog.value.id, message.sent_at)

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

  function handleMessageNew(event: WsEvent): void {
    // Support both formats:
    // 1. { payload: { message: {...} } } - expected format
    // 2. { id, dialog_id, sender_id, content, sent_at } - backend flat format
    let message: Message | undefined = event.payload?.message
    const msgType = (event.message_type || event.payload?.message_type || message?.message_type) as Message['message_type'] | undefined

    if (!message && event.id && event.dialog_id) {
      // Backend sends flat structure
      message = {
        id: event.id as string,
        dialog_id: event.dialog_id as string,
        sender_id: event.sender_id as string,
        content: event.content as string,
        sent_at: event.sent_at as string,
        message_type: msgType,
      }
    }

    if (!message) return

    // Update last_message_at for the dialog (even if not current dialog)
    updateDialogLastMessageAt(message.dialog_id, message.sent_at)

    // Increment unread_count for messages from other users (not system messages)
    const isFromOtherUser = message.sender_id !== config.userId
    const isSystemMessage = msgType === 'system'
    const isCurrentlyViewingDialog = currentDialog.value?.id === message.dialog_id

    if (isFromOtherUser && !isSystemMessage && !isCurrentlyViewingDialog) {
      // Increment unread in participating dialogs
      const activeIdx = participatingDialogs.value.findIndex((d) => d.id === message!.dialog_id)
      if (activeIdx !== -1) {
        participatingDialogs.value[activeIdx] = {
          ...participatingDialogs.value[activeIdx],
          unread_count: (participatingDialogs.value[activeIdx].unread_count || 0) + 1,
        }
      }

      // Increment unread in archived dialogs
      const archivedIdx = archivedDialogs.value.findIndex((d) => d.id === message!.dialog_id)
      if (archivedIdx !== -1) {
        archivedDialogs.value[archivedIdx] = {
          ...archivedDialogs.value[archivedIdx],
          unread_count: (archivedDialogs.value[archivedIdx].unread_count || 0) + 1,
        }
      }
    }

    // Only add message to local list if it's for the current dialog
    if (currentDialog.value && message.dialog_id === currentDialog.value.id) {
      // Check for duplicates
      if (!messages.value.some((m) => m.id === message!.id)) {
        messages.value = [...messages.value, message]
      }

      // If user is viewing this dialog and message is from another user,
      // mark as read after a short delay (for smart notifications)
      if (isFromOtherUser && !isSystemMessage) {
        setTimeout(() => {
          markAsRead(message!.id)
        }, 500)
      }
    }
  }

  function handleParticipantJoined(event: WsEvent): void {
    // Support both flat and payload formats
    const dialog_id = event.dialog_id || event.payload?.dialog_id
    const user_id = event.user_id || event.payload?.user_id
    if (!dialog_id || !user_id) return

    // Check if current user joined a new dialog - reload dialog lists
    if (user_id === config.userId) {
      // Current user joined a dialog - reload participating dialogs to show it
      loadParticipatingDialogs().catch(() => {})
      // Also reload available dialogs as dialog may no longer be "available"
      loadAvailableDialogs().catch(() => {})
      return
    }

    // Update participant count in dialogs (immutable update for reactivity)
    const updateCount = (dialogs: DialogListItem[]): DialogListItem[] => {
      const idx = dialogs.findIndex((d) => d.id === dialog_id)
      if (idx === -1) return dialogs
      return [
        ...dialogs.slice(0, idx),
        { ...dialogs[idx], participants_count: (dialogs[idx].participants_count || 0) + 1 },
        ...dialogs.slice(idx + 1),
      ]
    }

    participatingDialogs.value = updateCount(participatingDialogs.value)
    availableDialogs.value = updateCount(availableDialogs.value)

    if (currentDialog.value?.id === dialog_id) {
      currentDialog.value = {
        ...currentDialog.value,
        participants_count: (currentDialog.value.participants_count || 0) + 1,
      }
      // Reload participants list
      loadParticipants()
    }
  }

  function handleParticipantLeft(event: WsEvent): void {
    // Support both flat and payload formats
    const dialog_id = event.dialog_id || event.payload?.dialog_id
    const user_id = event.user_id || event.payload?.user_id
    if (!dialog_id || !user_id) return

    // Check if current user left a dialog - reload dialog lists
    if (user_id === config.userId) {
      // Current user left a dialog - reload to remove it from participating
      loadParticipatingDialogs().catch(() => {})
      // Also reload available dialogs as dialog may now be "available" again
      loadAvailableDialogs().catch(() => {})
      return
    }

    // Update participant count in dialogs (immutable update for reactivity)
    const updateCount = (dialogs: DialogListItem[]): DialogListItem[] => {
      const idx = dialogs.findIndex((d) => d.id === dialog_id)
      if (idx === -1) return dialogs
      const count = dialogs[idx].participants_count || 0
      return [
        ...dialogs.slice(0, idx),
        { ...dialogs[idx], participants_count: Math.max(0, count - 1) },
        ...dialogs.slice(idx + 1),
      ]
    }

    participatingDialogs.value = updateCount(participatingDialogs.value)
    availableDialogs.value = updateCount(availableDialogs.value)

    if (currentDialog.value?.id === dialog_id) {
      const count = currentDialog.value.participants_count || 0
      currentDialog.value = {
        ...currentDialog.value,
        participants_count: Math.max(0, count - 1),
      }
      // Reload participants list
      loadParticipants()
    }
  }

  function handleDialogArchived(event: WsEvent): void {
    const dialog_id = event.dialog_id || event.payload?.dialog_id
    if (!dialog_id) return

    // Move dialog from active to archived list
    const dialogIndex = participatingDialogs.value.findIndex((d) => d.id === dialog_id)

    if (dialogIndex !== -1) {
      const dialog = participatingDialogs.value[dialogIndex]
      // Remove from active list
      participatingDialogs.value = [
        ...participatingDialogs.value.slice(0, dialogIndex),
        ...participatingDialogs.value.slice(dialogIndex + 1),
      ]
      // Add to archived list with is_archived flag
      archivedDialogs.value = [{ ...dialog, is_archived: true }, ...archivedDialogs.value]
    } else {
      // Dialog not in participating list - reload to sync state
      loadParticipatingDialogs().catch(() => {})
      loadArchivedDialogs().catch(() => {})
    }

    // Update current dialog if it's the one being archived
    if (currentDialog.value?.id === dialog_id) {
      currentDialog.value = { ...currentDialog.value, is_archived: true }
    }
  }

  function handleDialogUnarchived(event: WsEvent): void {
    const dialog_id = event.dialog_id || event.payload?.dialog_id
    if (!dialog_id) return

    // Move dialog from archived to active list
    const dialogIndex = archivedDialogs.value.findIndex((d) => d.id === dialog_id)

    if (dialogIndex !== -1) {
      const dialog = archivedDialogs.value[dialogIndex]
      // Remove from archived list
      archivedDialogs.value = [
        ...archivedDialogs.value.slice(0, dialogIndex),
        ...archivedDialogs.value.slice(dialogIndex + 1),
      ]
      // Add to active list with is_archived flag removed
      participatingDialogs.value = [{ ...dialog, is_archived: false }, ...participatingDialogs.value]
    } else {
      // Dialog not found in archived list - reload to sync state
      loadParticipatingDialogs().catch(() => {})
      loadArchivedDialogs().catch(() => {})
    }

    // Update current dialog if it's the one being unarchived
    if (currentDialog.value?.id === dialog_id) {
      currentDialog.value = { ...currentDialog.value, is_archived: false }
    }
  }

  function handleMessageRead(event: WsEvent): void {
    // Support both flat and payload formats
    const dialog_id = event.dialog_id || event.payload?.dialog_id
    const readByUserId = event.user_id || event.payload?.user_id
    const lastReadMessageId = event.last_read_message_id || event.payload?.last_read_message_id

    if (!dialog_id || !readByUserId) return

    // Update participant's last_read_message_id in local state (for read receipts)
    if (currentDialog.value?.id === dialog_id && lastReadMessageId) {
      const participantIdx = participants.value.findIndex((p) => p.user_id === readByUserId)
      if (participantIdx !== -1) {
        participants.value = [
          ...participants.value.slice(0, participantIdx),
          { ...participants.value[participantIdx], last_read_message_id: lastReadMessageId },
          ...participants.value.slice(participantIdx + 1),
        ]
      }
    }

    // Only handle unread count if it's the current user's read receipt
    if (readByUserId === config.userId) {
      // Update unread count in active dialogs
      const activeIdx = participatingDialogs.value.findIndex((d) => d.id === dialog_id)
      if (activeIdx !== -1) {
        participatingDialogs.value[activeIdx] = {
          ...participatingDialogs.value[activeIdx],
          unread_count: 0,
        }
      }

      // Update unread count in archived dialogs
      const archivedIdx = archivedDialogs.value.findIndex((d) => d.id === dialog_id)
      if (archivedIdx !== -1) {
        archivedDialogs.value[archivedIdx] = {
          ...archivedDialogs.value[archivedIdx],
          unread_count: 0,
        }
      }

      // Update current dialog unread count (but keep divider)
      if (currentDialog.value?.id === dialog_id) {
        currentDialog.value = {
          ...currentDialog.value,
          unread_count: 0,
        }
        // NOTE: firstUnreadMessageId is NOT cleared
        // Divider stays visible until user re-enters the chat
      }
    }
  }

  function handlePresenceUpdate(event: WsEvent): void {
    // Support both flat and payload formats
    const userId = event.user_id || event.payload?.user_id
    const isOnline = event.is_online ?? event.payload?.is_online
    if (!userId || isOnline === undefined) return

    // Update onlineUsers set (immutable update for reactivity)
    const newSet = new Set(onlineUsers.value)
    if (isOnline) {
      newSet.add(userId)
    } else {
      newSet.delete(userId)
    }
    onlineUsers.value = newSet

    // Update participant in the list
    const idx = participants.value.findIndex((p) => p.user_id === userId)
    if (idx !== -1) {
      participants.value = [
        ...participants.value.slice(0, idx),
        { ...participants.value[idx], is_online: isOnline },
        ...participants.value.slice(idx + 1),
      ]
    }
  }

  function handleMessageEdited(event: WsEvent): void {
    const id = (event.id || event.payload?.id) as string | undefined
    const dialog_id = event.dialog_id || event.payload?.dialog_id
    const content = (event.content || event.payload?.content) as string | undefined
    const last_edited_at = (event.last_edited_at || event.payload?.last_edited_at) as string | undefined
    if (!id || !dialog_id || !content) return

    // Only update if it's for the current dialog
    if (currentDialog.value?.id !== dialog_id) return

    const idx = messages.value.findIndex((m) => m.id === id)
    if (idx !== -1) {
      messages.value = [
        ...messages.value.slice(0, idx),
        { ...messages.value[idx], content, last_edited_at },
        ...messages.value.slice(idx + 1),
      ]
    }
  }

  function handleMessageDeleted(event: WsEvent): void {
    const id = (event.id || event.payload?.id) as string | undefined
    const dialog_id = event.dialog_id || event.payload?.dialog_id
    if (!id || !dialog_id) return

    // Only update if it's for the current dialog
    if (currentDialog.value?.id !== dialog_id) return

    messages.value = messages.value.filter((m) => m.id !== id)
  }

  /**
   * Check if a user is currently online
   */
  function isUserOnline(userId: string): boolean {
    return onlineUsers.value.has(userId)
  }

  // ============ Lifecycle ============

  // Track if we've connected before (to distinguish reconnect from initial connect)
  let hasConnectedBefore = false

  onMounted(async () => {
    // Set up event handlers
    client.on('connected', async () => {
      isConnected.value = true

      // On reconnect, reload data that may have changed while disconnected
      if (hasConnectedBefore) {
        // Reload dialog lists
        loadParticipatingDialogs().catch(() => {})
        loadAvailableDialogs().catch(() => {})

        // Reload current dialog data
        if (currentDialog.value?.i_am_participant) {
          loadMessages().catch(() => {})
          loadParticipants().catch(() => {})
        }
      }

      hasConnectedBefore = true
    })

    client.on('disconnected', () => {
      isConnected.value = false
      // Clear online users - we don't know their status when disconnected
      onlineUsers.value = new Set()
    })

    // Real-time presence updates from server
    client.on('presence.update', handlePresenceUpdate)

    // Other events
    client.on('message.new', handleMessageNew)
    client.on('message.read', handleMessageRead)
    client.on('message.edited', handleMessageEdited)
    client.on('message.deleted', handleMessageDeleted)
    client.on('participant.joined', handleParticipantJoined)
    client.on('participant.left', handleParticipantLeft)
    client.on('dialog.archived', handleDialogArchived)
    client.on('dialog.unarchived', handleDialogUnarchived)

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
        await loadParticipants()
      }
    } else if (dialogId) {
      // Full mode with initial dialog
      await selectDialog(dialogId)
    }
  })

  onUnmounted(() => {
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
    archivedDialogs,
    participants,
    currentDialog,
    isConnected,
    isLoading,
    error,
    firstUnreadMessageId,
    replyToMessage,
    editingMessage,
    searchQuery,
    onlineUsers,

    // Reply cache and pagination state
    replyMessagesCache,
    hasMoreMessages,
    hasMoreAfter,
    isLoadingOlder,
    isLoadingNewer,
    isJumpingToMessage,
    jumpCooldown,

    // API access for file uploads
    api: client.api,

    // Methods
    connect,
    disconnect,
    sendMessage,
    editMessage,
    deleteMessage,
    loadMessages,
    loadOlderMessages,
    loadNewerMessages,
    resetToLatest,
    jumpToMessage,
    enableScrollCooldown,
    getReplyMessage,
    fetchReplyMessage,
    setSearchQuery,
    loadParticipatingDialogs,
    loadArchivedDialogs,
    loadAvailableDialogs,
    loadDialogByObject,
    selectDialog,
    joinDialog,
    leaveDialog,
    archiveDialog,
    unarchiveDialog,
    pinDialog,
    unpinDialog,
    toggleNotifications,
    subscribe,
    unsubscribe,
    markAsRead,
    setReplyTo,
    clearReplyTo,
    setEditMessage,
    clearEditMessage,
    isUserOnline,
  }
}

export type { UseChatOptions, UseChatReturn }
