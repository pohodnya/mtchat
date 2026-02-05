/**
 * MTChat SDK Client
 *
 * Combines REST API and WebSocket functionality
 */

import type { MTChatConfig, WsEvent, Message, JoinDialogRequest } from '../types'
import { MTChatApi } from './api'
import { MTChatWebSocket } from './websocket'

export type WsEventHandler = (event: WsEvent) => void

/**
 * Main SDK client combining API and WebSocket
 */
export class MTChatClient {
  public readonly api: MTChatApi
  private ws: MTChatWebSocket
  private config: MTChatConfig
  private eventHandlers: Map<string, Set<WsEventHandler>> = new Map()

  constructor(config: MTChatConfig) {
    this.config = config

    // Initialize REST API client
    this.api = new MTChatApi(config.baseUrl, config.userId, config.scopeConfig)

    // Derive WebSocket URL from base URL if not provided
    const wsUrl = config.wsUrl || this.deriveWsUrl(config.baseUrl)

    // Initialize WebSocket client
    this.ws = new MTChatWebSocket({
      url: `${wsUrl}?user_id=${config.userId}`,
      onConnect: () => {
        config.onConnect?.()
        this.emit({ type: 'connected' })
      },
      onDisconnect: () => {
        config.onDisconnect?.()
        this.emit({ type: 'disconnected' as any })
      },
      onError: (error) => {
        config.onError?.(error)
        this.emit({ type: 'error', payload: { error_message: error.message } })
      },
      onMessage: (event) => {
        config.onMessage?.(event)
        this.emit(event)
      },
      reconnect: config.reconnect,
      reconnectInterval: config.reconnectInterval,
      heartbeatInterval: config.heartbeatInterval,
    })
  }

  /**
   * Derive WebSocket URL from HTTP URL
   */
  private deriveWsUrl(baseUrl: string): string {
    const url = new URL(baseUrl)
    const wsProtocol = url.protocol === 'https:' ? 'wss:' : 'ws:'
    return `${wsProtocol}//${url.host}/api/v1/ws`
  }

  /**
   * Emit event to all handlers
   */
  private emit(event: WsEvent): void {
    // Emit to specific event handlers
    const handlers = this.eventHandlers.get(event.type)
    if (handlers) {
      handlers.forEach((handler) => {
        try {
          handler(event)
        } catch (e) {
          console.error('Error in event handler:', e)
        }
      })
    }

    // Emit to wildcard handlers
    const wildcardHandlers = this.eventHandlers.get('*')
    if (wildcardHandlers) {
      wildcardHandlers.forEach((handler) => {
        try {
          handler(event)
        } catch (e) {
          console.error('Error in wildcard handler:', e)
        }
      })
    }
  }

  /**
   * Connect to WebSocket server
   */
  connect(): void {
    this.ws.connect()
  }

  /**
   * Disconnect from WebSocket server
   */
  disconnect(): void {
    this.ws.disconnect()
  }

  /**
   * Check if connected to WebSocket
   */
  get isConnected(): boolean {
    return this.ws.isConnected
  }

  /**
   * Get current user ID
   */
  get userId(): string {
    return this.config.userId
  }

  /**
   * Subscribe to dialog updates
   */
  subscribe(dialogId: string): void {
    this.ws.subscribe(dialogId)
  }

  /**
   * Unsubscribe from dialog updates
   */
  unsubscribe(dialogId: string): void {
    this.ws.unsubscribe(dialogId)
  }

  /**
   * Add event listener
   * @returns Unsubscribe function
   */
  on(event: string, handler: WsEventHandler): () => void {
    if (!this.eventHandlers.has(event)) {
      this.eventHandlers.set(event, new Set())
    }
    this.eventHandlers.get(event)!.add(handler)

    return () => this.off(event, handler)
  }

  /**
   * Remove event listener
   */
  off(event: string, handler: WsEventHandler): void {
    this.eventHandlers.get(event)?.delete(handler)
  }

  /**
   * Send message (convenience method)
   */
  async sendMessage(dialogId: string, content: string): Promise<Message> {
    return this.api.sendMessage(dialogId, content)
  }

  /**
   * Join dialog (convenience method)
   */
  async joinDialog(dialogId: string, profile: JoinDialogRequest): Promise<void> {
    await this.api.joinDialog(dialogId, profile)
  }

  /**
   * Leave dialog (convenience method)
   */
  async leaveDialog(dialogId: string): Promise<void> {
    await this.api.leaveDialog(dialogId)
  }
}
