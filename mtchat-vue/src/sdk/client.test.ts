import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { MTChatClient } from './client'
import type { MTChatConfig } from '../types'

class FakeWebSocket {
  static readonly CONNECTING = 0
  static readonly OPEN = 1
  static readonly CLOSING = 2
  static readonly CLOSED = 3
  static instances: FakeWebSocket[] = []

  readyState = FakeWebSocket.CONNECTING
  sent: string[] = []
  onopen: (() => void) | null = null
  onclose: (() => void) | null = null
  onerror: (() => void) | null = null
  onmessage: ((event: { data: string }) => void) | null = null
  close = vi.fn(() => {
    this.readyState = FakeWebSocket.CLOSED
  })

  constructor(public url: string) {
    FakeWebSocket.instances.push(this)
  }

  send(data: string): void {
    this.sent.push(data)
  }

  open(): void {
    this.readyState = FakeWebSocket.OPEN
    this.onopen?.()
  }

  dropByServer(): void {
    this.readyState = FakeWebSocket.CLOSED
    this.onclose?.()
  }
}

const config: MTChatConfig = {
  baseUrl: 'https://chat.example.com',
  userId: 'user-1',
  token: 'old-token',
  scopeConfig: { scopeLevel0: ['tenant-1'], scopeLevel1: [], scopeLevel2: [] },
  userProfile: { displayName: 'User', company: 'Company A' },
  reconnectInterval: 1000,
}

function tokenOf(url: string): string | null {
  return new URL(url).searchParams.get('token')
}

beforeEach(() => {
  FakeWebSocket.instances = []
  vi.stubGlobal('WebSocket', FakeWebSocket)
})

afterEach(() => {
  vi.useRealTimers()
  vi.unstubAllGlobals()
  vi.restoreAllMocks()
})

describe('MTChatClient.setToken', () => {
  it('sends REST requests with the new token', async () => {
    const fetchMock = vi.fn(async (_url: string, _init: RequestInit) => new Response(JSON.stringify({ data: { count: 0 } }), { status: 200 }))
    vi.stubGlobal('fetch', fetchMock)
    const client = new MTChatClient(config)

    client.setToken('new-token')
    await client.api.getUnreadCount()

    const init = fetchMock.mock.calls[0][1]
    expect((init.headers as Record<string, string>)['Authorization']).toBe('Bearer new-token')
  })

  it('exposes the new token in client config without mutating the passed config', () => {
    const client = new MTChatClient(config)

    client.setToken('new-token')

    expect(client.config.token).toBe('new-token')
    expect(config.token).toBe('old-token')
  })

  it('keeps the open connection, subscriptions and handlers', () => {
    const client = new MTChatClient(config)
    const onDisconnected = vi.fn()
    const onMessage = vi.fn()
    client.on('disconnected', onDisconnected)
    client.on('message.new', onMessage)
    client.connect()
    const socket = FakeWebSocket.instances[0]
    socket.open()
    client.subscribe('dialog-1')

    client.setToken('new-token')
    socket.onmessage?.({ data: JSON.stringify({ type: 'message.new', payload: {} }) })

    expect(FakeWebSocket.instances).toHaveLength(1)
    expect(socket.close).not.toHaveBeenCalled()
    expect(client.isConnected).toBe(true)
    expect(onDisconnected).not.toHaveBeenCalled()
    expect(onMessage).toHaveBeenCalledTimes(1)
  })

  it('reconnects with the new token and resubscribes to dialogs', () => {
    vi.useFakeTimers()
    const client = new MTChatClient(config)
    client.connect()
    const socket = FakeWebSocket.instances[0]
    socket.open()
    client.subscribe('dialog-1')

    client.setToken('new-token')
    socket.dropByServer()
    vi.advanceTimersByTime(config.reconnectInterval!)
    const reconnected = FakeWebSocket.instances[1]
    reconnected.open()

    expect(tokenOf(socket.url)).toBe('old-token')
    expect(tokenOf(reconnected.url)).toBe('new-token')
    expect(reconnected.sent).toContain(JSON.stringify({ type: 'subscribe', dialog_id: 'dialog-1' }))
  })

  it('connects with the new token when set before the first connect', () => {
    const client = new MTChatClient(config)

    client.setToken('new-token')
    client.connect()

    expect(tokenOf(FakeWebSocket.instances[0].url)).toBe('new-token')
  })
})
