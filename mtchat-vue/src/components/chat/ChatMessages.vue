<script setup lang="ts">
/**
 * ChatMessages - Messages list with virtual scroll and infinite loading
 *
 * Handles:
 * - Message rendering (user/system)
 * - Virtual scrolling (vue-virtual-scroller) for 1000+ messages
 * - Infinite scroll (load older messages)
 * - Sticky date headers
 * - Unread message divider
 * - Scroll to bottom button
 * - Fallback to v-for when virtual scroller not installed or few messages
 */

import { ref, computed, watch, nextTick, onMounted, onUnmounted, shallowRef } from 'vue'
import type { Message, DialogParticipant, Attachment, VirtualItem } from '../../types'
import { useI18n } from '../../i18n'
import AttachmentList from './AttachmentList.vue'
import Icon from '../Icon.vue'

// Try to import vue-virtual-scroller (optional dependency)
// We use shallowRef to store components loaded at runtime
const DynamicScroller = shallowRef<any>(null)
const DynamicScrollerItem = shallowRef<any>(null)
const virtualScrollerLoaded = ref(false)

// Load virtual scroller on module init (non-blocking)
import('vue-virtual-scroller')
  .then((vvs) => {
    DynamicScroller.value = vvs.DynamicScroller
    DynamicScrollerItem.value = vvs.DynamicScrollerItem
    virtualScrollerLoaded.value = true
  })
  .catch(() => {
    // vue-virtual-scroller not installed, will use fallback
    virtualScrollerLoaded.value = true
  })

const props = defineProps<{
  messages: Message[]
  participants: DialogParticipant[]
  currentUserId: string
  firstUnreadMessageId: string | null
  isLoadingOlder: boolean
  isLoadingNewer: boolean
  hasMoreMessages: boolean
  hasMoreAfter: boolean
  isJumpingToMessage: boolean
  /** Cooldown after jump to prevent scroll cascade */
  jumpCooldown: boolean
  /** Cache for reply messages not in current list */
  replyMessagesCache: Map<string, Message | null>
}>()

const emit = defineEmits<{
  loadOlder: []
  loadNewer: []
  scrollToBottom: []
  resetToLatest: []
  reply: [message: Message]
  edit: [message: Message]
  openGallery: [attachment: Attachment]
  openFile: [attachment: Attachment]
  markAsRead: []
  jumpToMessage: [messageId: string]
}>()

// i18n
const { t, formatDateDivider } = useI18n()

// Refs
const containerRef = ref<HTMLElement | null>(null)
const scrollerRef = shallowRef<any>(null)

// State
const showScrollButton = ref(false)
const stickyDate = ref<string | null>(null)
const hiddenDividerDate = ref<string | null>(null)
const openMenuId = ref<string | null>(null)

// Mark as read timer
let readTimeout: ReturnType<typeof setTimeout> | null = null

// Virtual scroll threshold (use virtual scroll when message count exceeds this)
const VIRTUAL_SCROLL_THRESHOLD = 100

// Check if virtual scroller is available
const hasVirtualScroller = computed(() => DynamicScroller.value !== null)

// Use virtual scroll when available and message count exceeds threshold
const useVirtualScroll = computed(() => {
  return hasVirtualScroller.value && props.messages.length >= VIRTUAL_SCROLL_THRESHOLD
})

// ============ Virtual Items ============

/**
 * Transform messages array into flat list with dividers
 * for virtual scroller consumption
 */
const virtualItems = computed<VirtualItem[]>(() => {
  const items: VirtualItem[] = []
  let lastDateKey: string | null = null

  for (let i = 0; i < props.messages.length; i++) {
    const message = props.messages[i]
    const dateKey = getDateKey(message.sent_at)

    // Add date divider if date changed
    if (dateKey !== lastDateKey) {
      items.push({
        id: `date-${dateKey}`,
        type: 'date-divider',
        date: formatDateDivider(message.sent_at),
        dateKey: dateKey,
      })
      lastDateKey = dateKey
    }

    // Add unread divider before first unread message
    if (message.id === props.firstUnreadMessageId) {
      items.push({
        id: 'unread-divider',
        type: 'unread-divider',
      })
    }

    // Add message
    items.push({
      id: message.id,
      type: 'message',
      message: message,
    })
  }

  return items
})

// ============ Scroll Handling ============

function handleScroll() {
  const container = useVirtualScroll.value ? scrollerRef.value?.$el : containerRef.value
  if (!container) return

  const distanceFromBottom = container.scrollHeight - container.scrollTop - container.clientHeight
  const distanceFromTop = container.scrollTop
  const isAtBottom = distanceFromBottom < 50

  // Show/hide scroll to bottom button
  showScrollButton.value = distanceFromBottom > 200

  // Skip scroll-based loading during jump cooldown (prevents cascade)
  if (!props.jumpCooldown) {
    // Infinite scroll: load older messages when near top
    if (distanceFromTop < 200 && props.hasMoreMessages && !props.isLoadingOlder) {
      emit('loadOlder')
    }

    // Infinite scroll: load newer messages when near bottom (after jumping to a message)
    if (distanceFromBottom < 200 && props.hasMoreAfter && !props.isLoadingNewer) {
      emit('loadNewer')
    }
  }

  // Update sticky date
  updateStickyDate(container)

  // Mark as read when at bottom
  handleReadTracking(isAtBottom)
}

function updateStickyDate(container: HTMLElement) {
  const dateDividers = container.querySelectorAll('.chat-messages__date-divider')
  const containerRect = container.getBoundingClientRect()
  let activeDateText: string | null = null
  let hiddenDate: string | null = null
  let hideSticky = false

  const dividerArray = Array.from(dateDividers)

  for (let i = 0; i < dividerArray.length; i++) {
    const divider = dividerArray[i]
    const rect = divider.getBoundingClientRect()
    const relativeTop = rect.top - containerRect.top

    if (relativeTop < 0) {
      activeDateText = divider.textContent?.trim() || null
      hiddenDate = activeDateText

      const nextDivider = dividerArray[i + 1]
      if (nextDivider) {
        const nextRect = nextDivider.getBoundingClientRect()
        const nextRelativeTop = nextRect.top - containerRect.top
        if (nextRelativeTop >= 0 && nextRelativeTop < 60) {
          hideSticky = true
        }
      }
    }
  }

  stickyDate.value = hideSticky ? null : activeDateText
  hiddenDividerDate.value = hideSticky ? null : hiddenDate
}

function handleReadTracking(isAtBottom: boolean) {
  if (props.firstUnreadMessageId) {
    if (isAtBottom) {
      if (!readTimeout) {
        readTimeout = setTimeout(() => {
          emit('markAsRead')
          readTimeout = null
        }, 1000)
      }
    } else {
      if (readTimeout) {
        clearTimeout(readTimeout)
        readTimeout = null
      }
    }
  }
}

function scrollToBottom(smooth = false) {
  if (useVirtualScroll.value && scrollerRef.value) {
    // For virtual scroller, scroll to last item
    const lastIndex = virtualItems.value.length - 1
    if (lastIndex >= 0) {
      scrollerRef.value.scrollToItem(lastIndex)
      // After scrolling to item, ensure we're at the very bottom
      nextTick(() => {
        const el = scrollerRef.value?.$el
        if (el) {
          if (smooth) {
            el.scrollTo({ top: el.scrollHeight, behavior: 'smooth' })
          } else {
            el.scrollTop = el.scrollHeight
          }
        }
      })
    }
  } else if (containerRef.value) {
    if (smooth) {
      containerRef.value.scrollTo({
        top: containerRef.value.scrollHeight,
        behavior: 'smooth'
      })
    } else {
      containerRef.value.scrollTop = containerRef.value.scrollHeight
    }
  }
}

/**
 * Handle scroll-to-bottom button click
 * If there are more messages after current set, reload latest messages
 * Otherwise just scroll to bottom
 */
function handleScrollToBottom() {
  if (props.hasMoreAfter) {
    // We're not at the latest - need to load them
    emit('resetToLatest')
  } else {
    // Already at latest - just scroll
    emit('scrollToBottom')
    scrollToBottom(true)
  }
}

function scrollToMessage(messageId: string) {
  if (useVirtualScroll.value && scrollerRef.value) {
    // Find index of message in virtualItems
    const index = virtualItems.value.findIndex(item => item.id === messageId)
    if (index >= 0) {
      scrollerRef.value.scrollToItem(index)
      nextTick(() => {
        highlightMessage(messageId)
      })
    }
  } else {
    const messageEl = containerRef.value?.querySelector(`[data-message-id="${messageId}"]`)
    if (messageEl) {
      messageEl.scrollIntoView({ behavior: 'smooth', block: 'center' })
      highlightMessage(messageId)
    }
  }
}

function highlightMessage(messageId: string) {
  // Small delay to ensure element is in DOM after virtual scroll
  setTimeout(() => {
    const container = useVirtualScroll.value ? scrollerRef.value?.$el : containerRef.value
    const messageEl = container?.querySelector(`[data-message-id="${messageId}"]`)
    if (messageEl) {
      messageEl.classList.add('chat-messages__message--highlight')
      setTimeout(() => {
        messageEl.classList.remove('chat-messages__message--highlight')
      }, 5000)
    }
  }, 100)
}

// ============ Preserve scroll on prepend ============

// Store scroll position before loading older messages
let scrollHeightBefore = 0
let scrollTopBefore = 0
let isLoadingOlderLocal = false
let isLoadingNewerLocal = false

watch(() => props.isLoadingOlder, (loading) => {
  if (loading) {
    const container = useVirtualScroll.value ? scrollerRef.value?.$el : containerRef.value
    if (container) {
      // Starting to load - save position
      scrollHeightBefore = container.scrollHeight
      scrollTopBefore = container.scrollTop
      isLoadingOlderLocal = true
    }
  }
})

watch(() => props.isLoadingNewer, (loading) => {
  if (loading) {
    isLoadingNewerLocal = true
  }
})

watch(() => props.messages.length, async () => {
  if (isLoadingOlderLocal) {
    // Finished loading older messages - restore position
    await nextTick()
    const container = useVirtualScroll.value ? scrollerRef.value?.$el : containerRef.value
    if (container) {
      const scrollHeightAfter = container.scrollHeight
      container.scrollTop = scrollTopBefore + (scrollHeightAfter - scrollHeightBefore)
    }
    isLoadingOlderLocal = false
  } else if (isLoadingNewerLocal) {
    // Finished loading newer messages - don't scroll, just reset flag
    isLoadingNewerLocal = false
  } else if (!props.jumpCooldown) {
    // New real-time message added - scroll to bottom if we're near bottom
    // Skip if jumpCooldown is active (resetToLatest or jumpToMessage in progress)
    const container = useVirtualScroll.value ? scrollerRef.value?.$el : containerRef.value
    if (container) {
      const distanceFromBottom = container.scrollHeight - container.scrollTop - container.clientHeight
      // Only auto-scroll if user is already near bottom (< 100px away)
      if (distanceFromBottom < 100) {
        await nextTick()
        scrollToBottom()
      }
    }
  }
})

// ============ Message Helpers ============

function getDateKey(dateString: string): string {
  const date = new Date(dateString)
  return `${date.getFullYear()}-${date.getMonth()}-${date.getDate()}`
}

function shouldShowDateDivider(message: Message, index: number): boolean {
  if (index === 0) return true
  const prevMessage = props.messages[index - 1]
  return getDateKey(message.sent_at) !== getDateKey(prevMessage.sent_at)
}

function formatTime(dateString: string): string {
  const date = new Date(dateString)
  return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
}

function stripHtml(html: string): string {
  if (typeof document !== 'undefined') {
    const tmp = document.createElement('div')
    tmp.innerHTML = html
    return tmp.textContent || tmp.innerText || ''
  }
  return html.replace(/<[^>]*>/g, '')
}

function truncateText(text: string, maxLength: number): string {
  if (text.length <= maxLength) return text
  return text.slice(0, maxLength) + '...'
}

// ============ Participant/Sender Helpers ============

function getParticipant(userId: string): DialogParticipant | undefined {
  return props.participants.find(p => p.user_id === userId)
}

function getSenderDisplayName(senderId: string): string {
  const participant = getParticipant(senderId)
  if (participant?.display_name) return participant.display_name
  return senderId === props.currentUserId ? t.value.user.you : senderId.slice(0, 8)
}

function getSenderFullDisplay(senderId: string): string {
  const isCurrentUser = senderId === props.currentUserId
  const participant = getParticipant(senderId)

  let name = participant?.display_name || (isCurrentUser ? t.value.user.you : senderId.slice(0, 8))
  if (isCurrentUser) {
    name = `${name} ${t.value.user.youBadge}`
  }

  const company = participant?.company
  if (company) {
    return `${company} — ${name}`
  }
  return name
}

function getInitials(name: string): string {
  const parts = name.trim().split(/\s+/)
  if (parts.length >= 2) {
    return (parts[0][0] + parts[1][0]).toUpperCase()
  }
  return name.slice(0, 2).toUpperCase()
}

function isUserOnline(userId: string): boolean {
  const participant = getParticipant(userId)
  return participant?.is_online ?? false
}

// ============ Reply Helpers ============

function getReplyMessage(messageId: string): Message | null | undefined {
  // Check loaded messages first
  const loaded = props.messages.find(m => m.id === messageId)
  if (loaded) return loaded

  // Check cache
  if (props.replyMessagesCache.has(messageId)) {
    return props.replyMessagesCache.get(messageId)
  }

  return undefined
}

function getQuotedText(messageId: string): string {
  const msg = getReplyMessage(messageId)
  if (msg === undefined) return t.value.chat.messageLoading
  if (msg === null) return t.value.chat.messageDeleted
  const plainText = stripHtml(msg.content)
  return truncateText(plainText, 60)
}

function getMessageAuthor(messageId: string): string {
  const msg = getReplyMessage(messageId)
  if (msg === undefined) return t.value.chat.messageLoading
  if (msg === null) return ''
  if (!msg.sender_id) return ''
  return getSenderDisplayName(msg.sender_id)
}

/**
 * Handle click on quoted message
 * If message is loaded, scroll to it. Otherwise, emit jumpToMessage event.
 */
function handleQuotedClick(replyToId: string) {
  // Check if message is deleted
  const replyMsg = getReplyMessage(replyToId)
  if (replyMsg === null) {
    // Message was deleted, don't do anything
    return
  }

  // Check if message is in current list
  const loaded = props.messages.some(m => m.id === replyToId)
  if (loaded) {
    scrollToMessage(replyToId)
  } else {
    // Message not loaded, emit event to load it
    emit('jumpToMessage', replyToId)
  }
}

// ============ System Messages ============

interface SystemMessageContent {
  event: string
  participants?: Array<{ name: string; company?: string }>
  name?: string
  company?: string
}

function formatSystemMessage(message: Message): string {
  if (message.message_type !== 'system') return message.content

  try {
    const data: SystemMessageContent = JSON.parse(message.content)

    switch (data.event) {
      case 'chat_created': {
        const participants = data.participants
          ?.map(p => p.company ? `${p.name} (${p.company})` : p.name)
          .join(', ') || ''
        return t.value.system.chatCreated.replace('{participants}', participants)
      }
      case 'participant_joined': {
        const name = data.company
          ? `${data.name} (${data.company})`
          : data.name || ''
        return t.value.system.participantJoined.replace('{name}', name)
      }
      case 'participant_left': {
        return t.value.system.participantLeft.replace('{name}', data.name || '')
      }
      default:
        return message.content
    }
  } catch {
    return message.content
  }
}

// ============ Message Actions ============

function canEditMessage(message: Message): boolean {
  return message.sender_id === props.currentUserId && message.message_type !== 'system'
}

function toggleMessageMenu(messageId: string) {
  openMenuId.value = openMenuId.value === messageId ? null : messageId
}

function closeMessageMenu() {
  openMenuId.value = null
}

function handleReply(message: Message) {
  closeMessageMenu()
  emit('reply', message)
}

function handleEdit(message: Message) {
  closeMessageMenu()
  emit('edit', message)
}

function handleOpenGallery(message: Message, index: number) {
  const imageAttachments = message.attachments?.filter(a => a.content_type.startsWith('image/')) || []
  if (imageAttachments[index]) {
    emit('openGallery', imageAttachments[index])
  }
}

// ============ Virtual Scroll Size Dependencies ============

function getItemSizeDependencies(item: VirtualItem): any[] {
  if (item.type === 'message' && item.message) {
    return [
      item.message.content,
      item.message.attachments?.length ?? 0,
      item.message.reply_to_id ? 1 : 0,
    ]
  }
  return []
}

// ============ Click outside to close menu ============

function handleDocumentClick(e: MouseEvent) {
  if (openMenuId.value) {
    const menu = (e.target as Element).closest('.chat-messages__message-menu')
    const btn = (e.target as Element).closest('.chat-messages__action-btn')
    if (!menu && !btn) {
      closeMessageMenu()
    }
  }
}

onMounted(() => {
  document.addEventListener('click', handleDocumentClick)
  // Initial scroll to bottom
  nextTick(() => scrollToBottom())
})

onUnmounted(() => {
  document.removeEventListener('click', handleDocumentClick)
  if (readTimeout) {
    clearTimeout(readTimeout)
  }
})

// Expose for parent
defineExpose({
  scrollToBottom,
  scrollToMessage,
})
</script>

<template>
  <div class="chat-messages__wrapper">
    <!-- Floating sticky date (outside scroller for proper positioning) -->
    <div v-if="stickyDate" class="chat-messages__sticky-date">
      <span>{{ stickyDate }}</span>
    </div>

    <!-- Virtual scroll mode -->
    <component
      v-if="useVirtualScroll"
      :is="DynamicScroller"
      ref="scrollerRef"
      :items="virtualItems"
      :min-item-size="60"
      key-field="id"
      class="chat-messages"
      @scroll="handleScroll"
    >
      <template #default="{ item, active }">
        <component
          :is="DynamicScrollerItem"
          :item="item"
          :active="active"
          :size-dependencies="getItemSizeDependencies(item)"
        >
          <!-- Date divider -->
          <div
            v-if="item.type === 'date-divider'"
            :class="['chat-messages__date-divider', { 'chat-messages__date-divider--hidden': item.date === hiddenDividerDate }]"
          >
            <span>{{ item.date }}</span>
          </div>

          <!-- Unread divider -->
          <div
            v-else-if="item.type === 'unread-divider'"
            class="chat-messages__unread-divider"
          >
            <span>{{ t.chat.newMessages }}</span>
          </div>

          <!-- System message -->
          <div
            v-else-if="item.message?.message_type === 'system'"
            :data-message-id="item.message.id"
            class="chat-messages__system-message"
          >
            {{ formatSystemMessage(item.message) }}
          </div>

          <!-- User message -->
          <div
            v-else-if="item.message"
            :data-message-id="item.message.id"
            class="chat-messages__message"
          >
            <!-- Avatar -->
            <div class="chat-messages__avatar-wrapper">
              <div class="chat-messages__avatar">
                {{ item.message.sender_id ? getInitials(getSenderDisplayName(item.message.sender_id)) : '?' }}
              </div>
              <span
                v-if="item.message.sender_id && isUserOnline(item.message.sender_id)"
                class="chat-messages__avatar-online"
              ></span>
            </div>

            <!-- Message body -->
            <div class="chat-messages__body">
              <!-- Actions (visible on hover) -->
              <div class="chat-messages__actions">
                <button
                  class="chat-messages__action-btn"
                  :title="t.tooltips.menu"
                  @click.stop="toggleMessageMenu(item.message.id)"
                >
                  <Icon name="more-vertical" :size="16" />
                </button>

                <div v-if="openMenuId === item.message.id" class="chat-messages__message-menu">
                  <button @click.stop="handleReply(item.message)">
                    <Icon name="reply" :size="14" />
                    {{ t.actions.reply }}
                  </button>
                  <button v-if="canEditMessage(item.message)" @click.stop="handleEdit(item.message)">
                    <Icon name="edit" :size="14" />
                    {{ t.actions.edit }}
                  </button>
                </div>
              </div>

              <!-- Quoted message (if reply) -->
              <div
                v-if="item.message.reply_to_id"
                class="chat-messages__quoted"
                @click.stop="handleQuotedClick(item.message.reply_to_id)"
              >
                <div class="chat-messages__quoted-indicator"></div>
                <div class="chat-messages__quoted-content">
                  <div class="chat-messages__quoted-author">
                    {{ getMessageAuthor(item.message.reply_to_id) }}
                  </div>
                  <div class="chat-messages__quoted-text">
                    {{ getQuotedText(item.message.reply_to_id) }}
                  </div>
                </div>
              </div>

              <!-- Header -->
              <div class="chat-messages__header">
                <span class="chat-messages__sender">
                  {{ item.message.sender_id ? getSenderFullDisplay(item.message.sender_id) : '' }}
                </span>
                <span class="chat-messages__time">{{ formatTime(item.message.sent_at) }}</span>
                <span v-if="item.message.last_edited_at" class="chat-messages__edited">
                  ({{ t.chat.edited }})
                </span>
              </div>

              <!-- Content -->
              <div
                v-if="item.message.content"
                class="chat-messages__content"
                v-html="item.message.content"
              ></div>

              <!-- Attachments -->
              <AttachmentList
                v-if="item.message.attachments && item.message.attachments.length > 0"
                :attachments="item.message.attachments"
                @open-gallery="(idx) => handleOpenGallery(item.message!, idx)"
                @open-file="(att) => emit('openFile', att)"
              />
            </div>
          </div>
        </component>
      </template>
    </component>

    <!-- Fallback: standard v-for rendering -->
    <div v-else ref="containerRef" class="chat-messages" @scroll="handleScroll">
      <!-- Loading older messages indicator -->
      <div v-if="isLoadingOlder" class="chat-messages__loading-older">
        <span>{{ t.chat.loadingOlder }}</span>
      </div>

      <template v-for="(message, index) in messages" :key="message.id">
        <!-- Date divider -->
        <div
          v-if="shouldShowDateDivider(message, index)"
          :class="['chat-messages__date-divider', { 'chat-messages__date-divider--hidden': formatDateDivider(message.sent_at) === hiddenDividerDate }]"
        >
          <span>{{ formatDateDivider(message.sent_at) }}</span>
        </div>

        <!-- Unread divider -->
        <div
          v-if="message.id === firstUnreadMessageId"
          class="chat-messages__unread-divider"
        >
          <span>{{ t.chat.newMessages }}</span>
        </div>

        <!-- System message -->
        <div
          v-if="message.message_type === 'system'"
          :data-message-id="message.id"
          class="chat-messages__system-message"
        >
          {{ formatSystemMessage(message) }}
        </div>

        <!-- User message -->
        <div
          v-else
          :data-message-id="message.id"
          class="chat-messages__message"
        >
          <!-- Avatar -->
          <div class="chat-messages__avatar-wrapper">
            <div class="chat-messages__avatar">
              {{ message.sender_id ? getInitials(getSenderDisplayName(message.sender_id)) : '?' }}
            </div>
            <span
              v-if="message.sender_id && isUserOnline(message.sender_id)"
              class="chat-messages__avatar-online"
            ></span>
          </div>

          <!-- Message body -->
          <div class="chat-messages__body">
            <!-- Actions (visible on hover) -->
            <div class="chat-messages__actions">
              <button
                class="chat-messages__action-btn"
                :title="t.tooltips.menu"
                @click.stop="toggleMessageMenu(message.id)"
              >
                <Icon name="more-vertical" :size="16" />
              </button>

              <div v-if="openMenuId === message.id" class="chat-messages__message-menu">
                <button @click.stop="handleReply(message)">
                  <Icon name="reply" :size="14" />
                  {{ t.actions.reply }}
                </button>
                <button v-if="canEditMessage(message)" @click.stop="handleEdit(message)">
                  <Icon name="edit" :size="14" />
                  {{ t.actions.edit }}
                </button>
              </div>
            </div>

            <!-- Quoted message (if reply) -->
            <div
              v-if="message.reply_to_id"
              class="chat-messages__quoted"
              @click.stop="handleQuotedClick(message.reply_to_id)"
            >
              <div class="chat-messages__quoted-indicator"></div>
              <div class="chat-messages__quoted-content">
                <div class="chat-messages__quoted-author">
                  {{ getMessageAuthor(message.reply_to_id) }}
                </div>
                <div class="chat-messages__quoted-text">
                  {{ getQuotedText(message.reply_to_id) }}
                </div>
              </div>
            </div>

            <!-- Header -->
            <div class="chat-messages__header">
              <span class="chat-messages__sender">
                {{ message.sender_id ? getSenderFullDisplay(message.sender_id) : '' }}
              </span>
              <span class="chat-messages__time">{{ formatTime(message.sent_at) }}</span>
              <span v-if="message.last_edited_at" class="chat-messages__edited">
                ({{ t.chat.edited }})
              </span>
            </div>

            <!-- Content -->
            <div
              v-if="message.content"
              class="chat-messages__content"
              v-html="message.content"
            ></div>

            <!-- Attachments -->
            <AttachmentList
              v-if="message.attachments && message.attachments.length > 0"
              :attachments="message.attachments"
              @open-gallery="(idx) => handleOpenGallery(message, idx)"
              @open-file="(att) => emit('openFile', att)"
            />
          </div>
        </div>
      </template>

      <!-- Loading newer messages indicator -->
      <div v-if="isLoadingNewer" class="chat-messages__loading-newer">
        <span>{{ t.chat.loadingNewer }}</span>
      </div>

      <div v-if="messages.length === 0" class="chat-messages__empty">
        {{ t.chat.noMessages }}
      </div>
    </div>

    <!-- Loading overlay for jumping to message -->
    <div v-if="isJumpingToMessage" class="chat-messages__loading-overlay">
      <span>{{ t.chat.loadingOlder }}</span>
    </div>

    <!-- Loading older indicator (virtual scroll mode) -->
    <div v-if="useVirtualScroll && isLoadingOlder" class="chat-messages__loading-overlay">
      <span>{{ t.chat.loadingOlder }}</span>
    </div>

    <!-- Scroll to bottom button -->
    <button
      v-if="showScrollButton"
      class="chat-messages__scroll-btn"
      :title="t.tooltips.scrollDown"
      @click="handleScrollToBottom"
    >
      <Icon name="chevron-down" :size="18" />
    </button>
  </div>
</template>

<style scoped>
.chat-messages__wrapper {
  position: relative;
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.chat-messages {
  flex: 1;
  overflow-y: auto;
  padding: var(--mtchat-spacing-md, 12px);
  display: flex;
  flex-direction: column;
  gap: var(--mtchat-spacing-xs, 4px);
}

/* Sticky date */
.chat-messages__sticky-date {
  position: absolute;
  top: 8px;
  left: 50%;
  transform: translateX(-50%);
  z-index: 20;
  display: flex;
  justify-content: center;
  pointer-events: none;
}

.chat-messages__sticky-date span {
  padding: 4px 12px;
  background: var(--mtchat-bg);
  border: 1px solid var(--mtchat-border);
  border-radius: 12px;
  font-size: 12px;
  font-weight: 500;
  color: var(--mtchat-text-secondary);
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
  pointer-events: auto;
}

/* Loading older */
.chat-messages__loading-older {
  display: flex;
  justify-content: center;
  padding: 12px 0;
}

.chat-messages__loading-older span {
  padding: 6px 16px;
  background: var(--mtchat-bg-secondary);
  border-radius: 12px;
  font-size: 12px;
  color: var(--mtchat-text-secondary);
}

/* Loading newer */
.chat-messages__loading-newer {
  display: flex;
  justify-content: center;
  padding: 12px 0;
}

.chat-messages__loading-newer span {
  padding: 6px 16px;
  background: var(--mtchat-bg-secondary);
  border-radius: 12px;
  font-size: 12px;
  color: var(--mtchat-text-secondary);
}

/* Loading overlay for virtual scroll */
.chat-messages__loading-overlay {
  position: absolute;
  top: 8px;
  left: 50%;
  transform: translateX(-50%);
  z-index: 15;
  padding: 6px 16px;
  background: var(--mtchat-bg-secondary);
  border: 1px solid var(--mtchat-border);
  border-radius: 12px;
  font-size: 12px;
  color: var(--mtchat-text-secondary);
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

/* Date divider */
.chat-messages__date-divider {
  display: flex;
  justify-content: center;
  padding: 8px 0;
  margin: 8px 0;
}

.chat-messages__date-divider--hidden {
  visibility: hidden;
}

.chat-messages__date-divider span {
  padding: 4px 12px;
  background: var(--mtchat-bg-secondary);
  border-radius: 12px;
  font-size: 12px;
  font-weight: 500;
  color: var(--mtchat-text-secondary);
}

/* Unread divider */
.chat-messages__unread-divider {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 0;
}

.chat-messages__unread-divider::before,
.chat-messages__unread-divider::after {
  content: '';
  flex: 1;
  height: 1px;
  background: var(--mtchat-primary);
}

.chat-messages__unread-divider span {
  font-size: 12px;
  font-weight: 500;
  color: var(--mtchat-primary);
  white-space: nowrap;
}

/* System message */
.chat-messages__system-message {
  text-align: center;
  padding: 8px 16px;
  font-size: 12px;
  color: var(--mtchat-text-secondary);
}

/* User message */
.chat-messages__message {
  display: flex;
  gap: 12px;
  padding: 8px 12px;
  border-radius: 8px;
  transition: background-color 0.15s;
}

.chat-messages__message:hover {
  background: var(--mtchat-bg-hover);
}

.chat-messages__message--highlight {
  background: var(--mtchat-primary-bg, rgba(59, 130, 246, 0.1));
  animation: highlight-fade 4s ease-in;
}

@keyframes highlight-fade {
  0% { background: var(--mtchat-primary-bg, rgba(59, 130, 246, 0.2)); }
  100% { background: transparent; }
}

/* Avatar */
.chat-messages__avatar-wrapper {
  position: relative;
  flex-shrink: 0;
  width: 36px;
  height: 36px;
}

.chat-messages__avatar {
  width: 36px;
  height: 36px;
  border-radius: 50%;
  background: var(--mtchat-primary);
  color: white;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 12px;
  font-weight: 600;
}

.chat-messages__avatar-online {
  position: absolute;
  bottom: 0;
  right: 0;
  width: 10px;
  height: 10px;
  background: #22c55e;
  border: 2px solid var(--mtchat-bg);
  border-radius: 50%;
}

/* Message body */
.chat-messages__body {
  flex: 1;
  min-width: 0;
  position: relative;
}

.chat-messages__actions {
  position: absolute;
  top: 0;
  right: 0;
  opacity: 0;
  transition: opacity 0.15s;
}

.chat-messages__message:hover .chat-messages__actions {
  opacity: 1;
}

.chat-messages__action-btn {
  padding: 4px;
  background: var(--mtchat-bg);
  border: 1px solid var(--mtchat-border);
  border-radius: 4px;
  cursor: pointer;
  color: var(--mtchat-text-secondary);
}

.chat-messages__action-btn:hover {
  background: var(--mtchat-bg-hover);
  color: var(--mtchat-text);
}

.chat-messages__message-menu {
  position: absolute;
  top: 100%;
  right: 0;
  margin-top: 4px;
  background: var(--mtchat-bg);
  border: 1px solid var(--mtchat-border);
  border-radius: 8px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  z-index: 100;
  min-width: 140px;
  padding: 4px;
}

.chat-messages__message-menu button {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 8px 12px;
  background: none;
  border: none;
  font-size: 13px;
  color: var(--mtchat-text);
  cursor: pointer;
  border-radius: 4px;
}

.chat-messages__message-menu button:hover {
  background: var(--mtchat-bg-hover);
}

/* Quoted message */
.chat-messages__quoted {
  display: flex;
  gap: 8px;
  padding: 8px;
  margin-bottom: 4px;
  background: var(--mtchat-bg-secondary);
  border-radius: 6px;
  cursor: pointer;
}

.chat-messages__quoted:hover {
  background: var(--mtchat-bg-hover);
}

.chat-messages__quoted-indicator {
  width: 3px;
  background: var(--mtchat-primary);
  border-radius: 2px;
  flex-shrink: 0;
}

.chat-messages__quoted-content {
  min-width: 0;
}

.chat-messages__quoted-author {
  font-size: 12px;
  font-weight: 600;
  color: var(--mtchat-primary);
  margin-bottom: 2px;
}

.chat-messages__quoted-text {
  font-size: 12px;
  color: var(--mtchat-text-secondary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* Header */
.chat-messages__header {
  display: flex;
  align-items: baseline;
  gap: 8px;
  margin-bottom: 4px;
}

.chat-messages__sender {
  font-size: 13px;
  font-weight: 600;
  color: var(--mtchat-text);
}

.chat-messages__time {
  font-size: 11px;
  color: var(--mtchat-text-secondary);
}

.chat-messages__edited {
  font-size: 11px;
  color: var(--mtchat-text-secondary);
  font-style: italic;
}

/* Content */
.chat-messages__content {
  font-size: 14px;
  line-height: 1.5;
  color: var(--mtchat-text);
  word-break: break-word;
}

.chat-messages__content :deep(p) {
  margin: 0;
}

.chat-messages__content :deep(a) {
  color: var(--mtchat-primary);
  text-decoration: underline;
}

.chat-messages__content :deep(blockquote) {
  margin: 8px 0;
  padding-left: 12px;
  border-left: 3px solid var(--mtchat-border);
  color: var(--mtchat-text-secondary);
}

.chat-messages__content :deep(code) {
  padding: 2px 4px;
  background: var(--mtchat-bg-secondary);
  border-radius: 4px;
  font-family: monospace;
  font-size: 13px;
}

.chat-messages__content :deep(pre) {
  margin: 8px 0;
  padding: 12px;
  background: var(--mtchat-bg-secondary);
  border-radius: 6px;
  overflow-x: auto;
}

.chat-messages__content :deep(pre code) {
  padding: 0;
  background: none;
}

/* Empty state */
.chat-messages__empty {
  display: flex;
  align-items: center;
  justify-content: center;
  flex: 1;
  color: var(--mtchat-text-secondary);
  font-size: 14px;
}

/* Scroll button */
.chat-messages__scroll-btn {
  position: absolute;
  bottom: 16px;
  right: 16px;
  width: 40px;
  height: 40px;
  border-radius: 50%;
  background: var(--mtchat-bg);
  border: 1px solid var(--mtchat-border);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--mtchat-text-secondary);
  transition: all 0.15s;
}

.chat-messages__scroll-btn:hover {
  background: var(--mtchat-bg-hover);
  color: var(--mtchat-text);
}
</style>
