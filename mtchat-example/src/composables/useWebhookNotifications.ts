/**
 * Composable for receiving webhook notifications via SSE
 */

import { ref, onMounted, onUnmounted } from 'vue'

export interface WebhookEvent {
  type: string
  payload?: {
    dialog?: {
      id: string
      title: string
      object_type: string
      object_id: string
    }
    message?: {
      id: string
      content: string
      sent_at: string
    }
    sender?: {
      user_id: string
      display_name: string
      company: string
    }
    recipient_id?: string
    [key: string]: unknown
  }
}

export function useWebhookNotifications(serverUrl = 'http://localhost:3001') {
  const events = ref<WebhookEvent[]>([])
  const isConnected = ref(false)
  const lastEvent = ref<WebhookEvent | null>(null)

  let eventSource: EventSource | null = null

  function connect() {
    if (eventSource) {
      eventSource.close()
    }

    eventSource = new EventSource(`${serverUrl}/events`)

    eventSource.onopen = () => {
      console.log('[Webhook SSE] Connected')
      isConnected.value = true
    }

    eventSource.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data) as WebhookEvent
        console.log('[Webhook SSE] Received:', data.type)

        if (data.type !== 'connected') {
          events.value = [data, ...events.value.slice(0, 99)] // Keep last 100
          lastEvent.value = data
        }
      } catch (e) {
        console.error('[Webhook SSE] Parse error:', e)
      }
    }

    eventSource.onerror = () => {
      console.log('[Webhook SSE] Error/Disconnected')
      isConnected.value = false
      // Auto-reconnect after 3 seconds
      setTimeout(connect, 3000)
    }
  }

  function disconnect() {
    if (eventSource) {
      eventSource.close()
      eventSource = null
    }
    isConnected.value = false
  }

  function clearEvents() {
    events.value = []
    lastEvent.value = null
  }

  onMounted(() => {
    connect()
  })

  onUnmounted(() => {
    disconnect()
  })

  return {
    events,
    lastEvent,
    isConnected,
    connect,
    disconnect,
    clearEvents,
  }
}
