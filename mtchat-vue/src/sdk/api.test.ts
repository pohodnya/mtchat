import { describe, it, expect, vi, afterEach } from 'vitest'
import { MTChatApi } from './api'

const scopeConfig = { scopeLevel0: ['tenant-1'], scopeLevel1: ['dept-1'], scopeLevel2: [] }

function stubFetch(response: () => Response) {
  const fetchMock = vi.fn(async (_url: string, _init: RequestInit) => response())
  vi.stubGlobal('fetch', fetchMock)
  return fetchMock
}

afterEach(() => {
  vi.unstubAllGlobals()
})

describe('MTChatApi errors', () => {
  it.each([403, 404])('carries the HTTP status %i on the thrown error', async (status) => {
    stubFetch(() => new Response(JSON.stringify({ error: { message: 'No access' } }), { status }))
    const api = new MTChatApi('https://chat.example.com', 'user-1', scopeConfig, 'token-1')

    await expect(api.getDialog('dialog-1')).rejects.toMatchObject({ status, message: 'No access' })
  })
})

describe('MTChatApi.getUnreadCount', () => {
  it('requests the unread count in the client scope and returns it', async () => {
    const fetchMock = stubFetch(() => new Response(JSON.stringify({ data: { count: 12 } }), { status: 200 }))
    const api = new MTChatApi('https://chat.example.com', 'user-1', scopeConfig, 'token-1')

    const count = await api.getUnreadCount()

    expect(count).toBe(12)
    const [url, init] = fetchMock.mock.calls[0]
    expect(new URL(url).pathname).toBe('/api/v1/unread')
    expect(init.method).toBe('GET')
    const headers = init.headers as Record<string, string>
    expect(JSON.parse(atob(headers['X-Scope-Config']))).toEqual({
      scope_level0: ['tenant-1'],
      scope_level1: ['dept-1'],
      scope_level2: [],
    })
    expect(headers['Authorization']).toBe('Bearer token-1')
  })
})
