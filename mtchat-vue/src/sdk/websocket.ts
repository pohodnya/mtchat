/**
 * MTChat WebSocket Client
 *
 * Handles real-time communication with the chat server.
 */

import type { WsEvent, WsClientMessage } from '../types'

export type WsEventHandler = (event: WsEvent) => void

export interface WsClientOptions {
  /** WebSocket URL (with query parameters already included) */
  url: string
  onConnect?: () => void
  onDisconnect?: () => void
  onError?: (error: Error) => void
  onMessage?: WsEventHandler
  /** Auto-reconnect on disconnect (default: true) */
  reconnect?: boolean
  /** Reconnect interval in ms (default: 3000) */
  reconnectInterval?: number
  /** Heartbeat interval in ms (default: 30000) */
  heartbeatInterval?: number
}

export class MTChatWebSocket {
  private ws: WebSocket | null = null
  private options: Required<WsClientOptions>
  private reconnectTimer: ReturnType<typeof setTimeout> | null = null
  private heartbeatTimer: ReturnType<typeof setInterval> | null = null
  private isManualClose = false
  private subscribedDialogs: Set<string> = new Set()

  constructor(options: WsClientOptions) {
    this.options = {
      reconnect: true,
      reconnectInterval: 3000,
      heartbeatInterval: 10000, // 10 seconds for faster presence updates
      onConnect: () => {},
      onDisconnect: () => {},
      onError: () => {},
      onMessage: () => {},
      ...options,
    }
  }

  connect(): void {
    if (this.ws?.readyState === WebSocket.OPEN) {
      return
    }

    this.isManualClose = false

    try {
      this.ws = new WebSocket(this.options.url)
      this.setupEventHandlers()
    } catch (error) {
      this.options.onError(error as Error)
      this.scheduleReconnect()
    }
  }

  disconnect(): void {
    this.isManualClose = true
    this.clearTimers()
    this.subscribedDialogs.clear()

    if (this.ws) {
      this.ws.close()
      this.ws = null
    }
  }

  send(message: WsClientMessage): void {
    if (this.ws?.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(message))
    }
  }

  subscribe(dialogId: string): void {
    this.subscribedDialogs.add(dialogId)
    this.send({ type: 'subscribe', dialog_id: dialogId })
  }

  unsubscribe(dialogId: string): void {
    this.subscribedDialogs.delete(dialogId)
    this.send({ type: 'unsubscribe', dialog_id: dialogId })
  }

  ping(): void {
    this.send({ type: 'ping' })
  }

  get isConnected(): boolean {
    return this.ws?.readyState === WebSocket.OPEN
  }

  private setupEventHandlers(): void {
    if (!this.ws) return

    this.ws.onopen = () => {
      this.options.onConnect()
      this.startHeartbeat()
      // Resubscribe to dialogs after reconnect
      this.subscribedDialogs.forEach((dialogId) => {
        this.send({ type: 'subscribe', dialog_id: dialogId })
      })
    }

    this.ws.onclose = () => {
      this.options.onDisconnect()
      this.clearTimers()

      if (!this.isManualClose && this.options.reconnect) {
        this.scheduleReconnect()
      }
    }

    this.ws.onerror = (_event) => {
      this.options.onError(new Error('WebSocket error'))
    }

    this.ws.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data) as WsEvent
        this.options.onMessage(data)
      } catch (error) {
        this.options.onError(new Error('Failed to parse WebSocket message'))
      }
    }
  }

  private startHeartbeat(): void {
    this.clearHeartbeat()
    this.heartbeatTimer = setInterval(() => {
      this.ping()
    }, this.options.heartbeatInterval)
  }

  private clearHeartbeat(): void {
    if (this.heartbeatTimer) {
      clearInterval(this.heartbeatTimer)
      this.heartbeatTimer = null
    }
  }

  private scheduleReconnect(): void {
    this.clearReconnectTimer()
    this.reconnectTimer = setTimeout(() => {
      this.connect()
    }, this.options.reconnectInterval)
  }

  private clearReconnectTimer(): void {
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer)
      this.reconnectTimer = null
    }
  }

  private clearTimers(): void {
    this.clearHeartbeat()
    this.clearReconnectTimer()
  }
}
