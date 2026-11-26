import { describe, it, expect, vi, afterEach, onTestFinished } from 'vitest'
import { defineComponent, h, isRef, ref, shallowRef, type Ref } from 'vue'
import { mount, flushPromises, type VueWrapper } from '@vue/test-utils'
import { useChat } from './useChat'
import { MTChatClient } from '../sdk/client'
import { MTChatApi } from '../sdk/api'
import type { ChatMode, DialogListItem, Message, MTChatConfig, UseChatReturn, WsEvent } from '../types'

const config: MTChatConfig = {
  baseUrl: 'https://chat.example.com',
  userId: 'user-1',
  scopeConfig: { scopeLevel0: ['tenant-1'], scopeLevel1: [], scopeLevel2: [] },
  userProfile: { displayName: 'User', company: 'Company A' },
}

function buildDialog(id: string): DialogListItem {
  return {
    id,
    object_id: 'object-1',
    object_type: 'order',
    created_at: '2026-01-01T00:00:00Z',
    participants_count: 2,
    i_am_participant: true,
    unread_count: 0,
  }
}

function buildMessage(id: string, dialogId: string): Message {
  return {
    id,
    dialog_id: dialogId,
    sender_id: 'user-2',
    content: 'Hello',
    sent_at: '2026-01-01T00:00:00Z',
  }
}

function createClient({ connected = false, dialogs = [] as DialogListItem[] } = {}): MTChatClient {
  const client = new MTChatClient(config)
  vi.spyOn(client, 'connect').mockImplementation(() => {})
  vi.spyOn(client, 'disconnect').mockImplementation(() => {})
  vi.spyOn(client, 'isConnected', 'get').mockReturnValue(connected)
  vi.spyOn(client.api, 'getParticipatingDialogs').mockImplementation(async (_search, archived) =>
    archived ? [] : dialogs
  )
  vi.spyOn(client.api, 'getAvailableDialogs').mockResolvedValue([])
  vi.spyOn(client.api, 'getDialogsByObject').mockResolvedValue([])
  vi.spyOn(client.api, 'getDialog').mockImplementation(async (id) => buildDialog(id))
  vi.spyOn(client.api, 'getMessages').mockResolvedValue({ messages: [] })
  vi.spyOn(client.api, 'getParticipants').mockResolvedValue([])
  vi.spyOn(client.api, 'archiveDialog').mockResolvedValue()
  vi.spyOn(client.api, 'unarchiveDialog').mockResolvedValue()
  return client
}

function emitEvent(client: MTChatClient, event: WsEvent): void {
  (client as unknown as { emit(event: WsEvent): void }).emit(event)
}

function httpError(status: number): Error {
  return Object.assign(new Error(`HTTP ${status}`), { status })
}

function deferred<T>(): { promise: Promise<T>; resolve: (value: T) => void; reject: (reason: unknown) => void } {
  let resolve!: (value: T) => void
  let reject!: (reason: unknown) => void
  const promise = new Promise<T>((res, rej) => { resolve = res; reject = rej })
  return { promise, resolve, reject }
}

const wrappers: VueWrapper[] = []

interface MountOptions {
  client?: Ref<MTChatClient | null>
  config?: Ref<MTChatConfig>
  mode?: ChatMode
  objectType?: string
  objectId?: string | Ref<string | undefined>
  dialogId?: Ref<string | undefined>
}

async function mountChat(options: MountOptions): Promise<{ chat: UseChatReturn; wrapper: VueWrapper }> {
  let chat!: UseChatReturn
  const wrapper = mount(defineComponent({
    setup() {
      chat = useChat({
        ...(options.client ? { client: options.client } : { config: options.config }),
        mode: options.mode ?? 'full',
        dialogId: options.dialogId ?? ref(undefined),
        objectId: isRef(options.objectId) ? options.objectId : ref(options.objectId),
        objectType: ref(options.objectType),
      })
      return () => h('div')
    },
  }))
  wrappers.push(wrapper)
  await flushPromises()
  return { chat, wrapper }
}

afterEach(() => {
  wrappers.splice(0).forEach((wrapper) => wrapper.unmount())
  vi.restoreAllMocks()
})

describe('useChat with external client', () => {
  it('does not connect or disconnect the client', async () => {
    const client = createClient()

    const { wrapper } = await mountChat({ client: shallowRef(client) })
    wrapper.unmount()

    expect(client.connect).not.toHaveBeenCalled()
    expect(client.disconnect).not.toHaveBeenCalled()
  })

  it('leaves the shared connection alone on connect and disconnect', async () => {
    const client = createClient({ connected: true })
    const { chat } = await mountChat({ client: shallowRef(client) })

    chat.connect()
    chat.disconnect()

    expect(client.connect).not.toHaveBeenCalled()
    expect(client.disconnect).not.toHaveBeenCalled()
  })

  it('uses the client set between setup and mount', async () => {
    const oldClient = createClient({ connected: true })
    const newClient = createClient({ connected: true, dialogs: [buildDialog('dialog-1')] })
    const clientRef = shallowRef<MTChatClient | null>(oldClient)
    let chat!: UseChatReturn
    wrappers.push(mount(defineComponent({
      setup() {
        chat = useChat({ client: clientRef, mode: 'full', dialogId: ref(undefined), objectId: ref(), objectType: ref() })
        clientRef.value = newClient
        return () => h('div')
      },
    })))
    await flushPromises()

    expect(newClient.api.getParticipatingDialogs).toHaveBeenCalledTimes(1)
    expect(oldClient.api.getParticipatingDialogs).not.toHaveBeenCalled()
    emitEvent(newClient, { type: 'message.new', payload: { message: buildMessage('message-1', 'dialog-1') } })
    expect(chat.participatingDialogs.value[0].unread_count).toBe(1)
  })

  it('stops reacting to client events after unmount', async () => {
    const client = createClient({ connected: true, dialogs: [buildDialog('dialog-1')] })
    const { chat, wrapper } = await mountChat({ client: shallowRef(client) })

    wrapper.unmount()
    emitEvent(client, { type: 'message.new', payload: { message: buildMessage('message-1', 'dialog-1') } })
    emitEvent(client, { type: 'disconnected' })

    expect(chat.participatingDialogs.value[0].unread_count).toBe(0)
    expect(chat.isConnected.value).toBe(true)
  })

  it('loads data right away when the client is already connected', async () => {
    const client = createClient({ connected: true })

    const { chat } = await mountChat({ client: shallowRef(client) })

    expect(chat.isConnected.value).toBe(true)
    expect(client.api.getParticipatingDialogs).toHaveBeenCalledTimes(1)
    expect(client.api.getAvailableDialogs).toHaveBeenCalledTimes(1)
  })

  it('follows the new client and ignores the old one when the client is replaced', async () => {
    const oldClient = createClient({ connected: true, dialogs: [buildDialog('dialog-1')] })
    const newClient = createClient({ connected: true, dialogs: [buildDialog('dialog-1')] })
    const clientRef = shallowRef<MTChatClient | null>(oldClient)
    const { chat } = await mountChat({ client: clientRef })

    clientRef.value = newClient
    await flushPromises()

    expect(newClient.api.getParticipatingDialogs).toHaveBeenCalledTimes(1)
    expect(newClient.api.getAvailableDialogs).toHaveBeenCalledTimes(1)

    emitEvent(oldClient, { type: 'message.new', payload: { message: buildMessage('message-1', 'dialog-1') } })
    emitEvent(oldClient, { type: 'disconnected' })
    expect(chat.participatingDialogs.value[0].unread_count).toBe(0)
    expect(chat.isConnected.value).toBe(true)

    emitEvent(newClient, { type: 'message.new', payload: { message: buildMessage('message-2', 'dialog-1') } })
    emitEvent(newClient, { type: 'disconnected' })
    expect(chat.participatingDialogs.value[0].unread_count).toBe(1)
    expect(chat.isConnected.value).toBe(false)
  })

  it('reloads the open dialog through the new client when the client is replaced', async () => {
    const oldClient = createClient({ connected: true, dialogs: [buildDialog('dialog-1')] })
    const newClient = createClient({ connected: true, dialogs: [buildDialog('dialog-1')] })
    vi.mocked(newClient.api.getMessages).mockResolvedValue({ messages: [buildMessage('message-1', 'dialog-1')] })
    const clientRef = shallowRef<MTChatClient | null>(oldClient)
    const { chat } = await mountChat({ client: clientRef })
    await chat.selectDialog('dialog-1')

    clientRef.value = newClient
    await flushPromises()

    expect(chat.currentDialog.value?.id).toBe('dialog-1')
    expect(newClient.api.getMessages).toHaveBeenCalledWith('dialog-1', expect.anything(), expect.anything())
    expect(newClient.api.getParticipants).toHaveBeenCalledWith('dialog-1', expect.anything())
    expect(chat.messages.value.map((m) => m.id)).toEqual(['message-1'])
  })

  it.each([403, 404])('closes the open dialog the new client answers %i for', async (status) => {
    const oldClient = createClient({ connected: true, dialogs: [buildDialog('dialog-1')] })
    vi.mocked(oldClient.api.getMessages).mockResolvedValue({ messages: [buildMessage('message-1', 'dialog-1')] })
    const newClient = createClient({ connected: true })
    vi.mocked(newClient.api.getDialog).mockRejectedValue(httpError(status))
    const clientRef = shallowRef<MTChatClient | null>(oldClient)
    const { chat } = await mountChat({ client: clientRef })
    await chat.selectDialog('dialog-1')

    clientRef.value = newClient
    await flushPromises()

    expect(chat.currentDialog.value).toBeNull()
    expect(chat.messages.value).toEqual([])
    expect(chat.participants.value).toEqual([])
    expect(chat.error.value).toBeNull()
    expect(newClient.api.getMessages).not.toHaveBeenCalled()
  })

  it('keeps the loading indicator on until the new client loads its list', async () => {
    const oldClient = createClient({ connected: true })
    const oldList = deferred<DialogListItem[]>()
    vi.mocked(oldClient.api.getParticipatingDialogs).mockReturnValue(oldList.promise)
    const newClient = createClient({ connected: true })
    const newList = deferred<DialogListItem[]>()
    vi.mocked(newClient.api.getParticipatingDialogs).mockReturnValue(newList.promise)
    const clientRef = shallowRef<MTChatClient | null>(oldClient)
    const { chat } = await mountChat({ client: clientRef })

    clientRef.value = newClient
    await flushPromises()
    oldList.resolve([])
    await flushPromises()

    expect(chat.isLoadingDialogs.value).toBe(true)

    newList.resolve([])
    await flushPromises()

    expect(chat.isLoadingDialogs.value).toBe(false)
  })

  it('ignores a late dialog list of the previous object', async () => {
    const client = createClient({ connected: true })
    const lateList = deferred<DialogListItem[]>()
    vi.mocked(client.api.getDialogsByObject).mockImplementation(async ({ objectId, archived }) => {
      if (archived) return []
      return objectId === 'a' ? lateList.promise : [buildDialog('dialog-of-b')]
    })
    const objectIdRef = ref<string | undefined>('a')
    const { chat } = await mountChat({ client: shallowRef(client), objectType: 'order', objectId: objectIdRef })

    objectIdRef.value = 'b'
    await flushPromises()
    lateList.resolve([buildDialog('dialog-of-a')])
    await flushPromises()

    expect(chat.participatingDialogs.value.map((d) => d.id)).toEqual(['dialog-of-b'])
  })

  it('ignores a late dialog list from the replaced client', async () => {
    const oldClient = createClient({ connected: true })
    const lateList = deferred<DialogListItem[]>()
    vi.mocked(oldClient.api.getParticipatingDialogs).mockReturnValue(lateList.promise)
    const newClient = createClient({ connected: true, dialogs: [buildDialog('dialog-new')] })
    const clientRef = shallowRef<MTChatClient | null>(oldClient)
    const { chat } = await mountChat({ client: clientRef })

    clientRef.value = newClient
    await flushPromises()
    lateList.resolve([buildDialog('dialog-old')])
    await flushPromises()

    expect(chat.participatingDialogs.value.map((d) => d.id)).toEqual(['dialog-new'])
  })

  it('clears dialogs and messages when the client is removed', async () => {
    const client = createClient({ connected: true, dialogs: [buildDialog('dialog-1')] })
    vi.mocked(client.api.getMessages).mockResolvedValue({ messages: [buildMessage('message-1', 'dialog-1')] })
    const clientRef = shallowRef<MTChatClient | null>(client)
    const { chat } = await mountChat({ client: clientRef })
    await chat.selectDialog('dialog-1')

    clientRef.value = null
    await flushPromises()

    expect(chat.currentDialog.value).toBeNull()
    expect(chat.messages.value).toEqual([])
    expect(chat.participatingDialogs.value).toEqual([])
    expect(chat.isConnected.value).toBe(false)
    expect(chat.api).toBeNull()
  })

  it('waits for a client before loading data', async () => {
    const client = createClient({ connected: true })
    const clientRef = shallowRef<MTChatClient | null>(null)
    const { chat } = await mountChat({ client: clientRef })

    clientRef.value = client
    await flushPromises()

    expect(chat.isConnected.value).toBe(true)
    expect(client.api.getParticipatingDialogs).toHaveBeenCalledTimes(1)
  })

  it('keeps two instances on one client independent', async () => {
    const client = createClient({ connected: true })
    vi.mocked(client.api.getDialogsByObject).mockImplementation(async ({ objectId, archived }) =>
      archived ? [] : [buildDialog(`dialog-of-${objectId}`)]
    )
    const first = await mountChat({ client: shallowRef(client), objectType: 'order', objectId: 'a' })
    const second = await mountChat({ client: shallowRef(client), objectType: 'order', objectId: 'b' })

    emitEvent(client, { type: 'message.new', payload: { message: buildMessage('message-1', 'dialog-of-a') } })

    expect(first.chat.participatingDialogs.value[0].unread_count).toBe(1)
    expect(second.chat.participatingDialogs.value[0].unread_count).toBe(0)

    first.wrapper.unmount()
    emitEvent(client, { type: 'message.new', payload: { message: buildMessage('message-2', 'dialog-of-b') } })

    expect(second.chat.participatingDialogs.value[0].unread_count).toBe(1)
    expect(client.disconnect).not.toHaveBeenCalled()
  })

  it('does not subscribe to dialogs on the shared client', async () => {
    const client = createClient({ connected: true })
    const subscribe = vi.spyOn(client, 'subscribe')
    const unsubscribe = vi.spyOn(client, 'unsubscribe')
    const { chat } = await mountChat({ client: shallowRef(client) })

    chat.subscribe('dialog-1')
    chat.unsubscribe('dialog-1')

    expect(subscribe).not.toHaveBeenCalled()
    expect(unsubscribe).not.toHaveBeenCalled()
  })

  it('keeps the loaded dialogs and the open dialog when the token is refreshed', async () => {
    const client = createClient({ connected: true, dialogs: [buildDialog('dialog-1')] })
    vi.mocked(client.api.getMessages).mockResolvedValue({ messages: [buildMessage('message-1', 'dialog-1')] })
    const { chat } = await mountChat({ client: shallowRef(client) })
    await chat.selectDialog('dialog-1')
    vi.mocked(client.api.getParticipatingDialogs).mockClear()
    vi.mocked(client.api.getAvailableDialogs).mockClear()
    vi.mocked(client.api.getMessages).mockClear()

    client.setToken('new-token')
    await flushPromises()

    expect(client.api.getParticipatingDialogs).not.toHaveBeenCalled()
    expect(client.api.getAvailableDialogs).not.toHaveBeenCalled()
    expect(client.api.getMessages).not.toHaveBeenCalled()
    expect(chat.currentDialog.value?.id).toBe('dialog-1')
    expect(chat.messages.value.map((message) => message.id)).toEqual(['message-1'])
    expect(chat.isLoading.value).toBe(false)
  })
})

describe('useChat after a client swap', () => {
  function archiveOf(client: MTChatClient, dialogs: DialogListItem[]): void {
    vi.mocked(client.api.getParticipatingDialogs).mockImplementation(async (_search, archived) =>
      archived ? dialogs : []
    )
  }

  it('reloads the requested archive through the new client', async () => {
    const oldClient = createClient({ connected: true })
    archiveOf(oldClient, [buildDialog('dialog-old')])
    const newClient = createClient({ connected: true })
    archiveOf(newClient, [buildDialog('dialog-new')])
    const clientRef = shallowRef<MTChatClient | null>(oldClient)
    const { chat } = await mountChat({ client: clientRef })
    await chat.loadArchivedDialogs()

    clientRef.value = newClient
    await flushPromises()

    expect(newClient.api.getParticipatingDialogs).toHaveBeenCalledWith(undefined, true)
    expect(chat.archivedDialogs.value.map((d) => d.id)).toEqual(['dialog-new'])
  })

  it('reloads the requested archive of the object through the new client', async () => {
    const oldClient = createClient({ connected: true })
    vi.mocked(oldClient.api.getDialogsByObject).mockImplementation(async ({ archived }) =>
      archived ? [buildDialog('dialog-old')] : []
    )
    const newClient = createClient({ connected: true })
    vi.mocked(newClient.api.getDialogsByObject).mockImplementation(async ({ archived }) =>
      archived ? [buildDialog('dialog-new')] : []
    )
    const clientRef = shallowRef<MTChatClient | null>(oldClient)
    const { chat } = await mountChat({ client: clientRef, objectType: 'order', objectId: 'object-1' })
    await chat.loadArchivedDialogsByObject('order', 'object-1')

    clientRef.value = newClient
    await flushPromises()

    expect(chat.archivedDialogs.value.map((d) => d.id)).toEqual(['dialog-new'])
    expect(newClient.api.getParticipatingDialogs).not.toHaveBeenCalled()
  })

  it('drops the archive of the previous client without loading one nobody asked for', async () => {
    const oldClient = createClient({ connected: true, dialogs: [buildDialog('dialog-old')] })
    const newClient = createClient({ connected: true })
    const clientRef = shallowRef<MTChatClient | null>(oldClient)
    const { chat } = await mountChat({ client: clientRef })
    await chat.archiveDialog('dialog-old')
    expect(chat.archivedDialogs.value.map((d) => d.id)).toEqual(['dialog-old'])

    clientRef.value = newClient
    await flushPromises()

    expect(chat.archivedDialogs.value).toEqual([])
    expect(newClient.api.getParticipatingDialogs).not.toHaveBeenCalledWith(undefined, true)
  })

  it('reloads the requested archive when a client is set after removal', async () => {
    const oldClient = createClient({ connected: true })
    archiveOf(oldClient, [buildDialog('dialog-old')])
    const newClient = createClient({ connected: true })
    archiveOf(newClient, [buildDialog('dialog-new')])
    const clientRef = shallowRef<MTChatClient | null>(oldClient)
    const { chat } = await mountChat({ client: clientRef })
    await chat.loadArchivedDialogs()

    clientRef.value = null
    await flushPromises()
    expect(chat.archivedDialogs.value).toEqual([])

    clientRef.value = newClient
    await flushPromises()

    expect(chat.archivedDialogs.value.map((d) => d.id)).toEqual(['dialog-new'])
  })

  it('stops the loading indicator when the new client closes a dialog whose messages were loading', async () => {
    const oldClient = createClient({ connected: true, dialogs: [buildDialog('dialog-1')] })
    vi.mocked(oldClient.api.getMessages).mockReturnValue(new Promise(() => {}))
    const newClient = createClient({ connected: true })
    vi.mocked(newClient.api.getDialog).mockRejectedValue(httpError(403))
    const clientRef = shallowRef<MTChatClient | null>(oldClient)
    const { chat } = await mountChat({ client: clientRef })
    chat.selectDialog('dialog-1')
    await flushPromises()
    expect(chat.isLoading.value).toBe(true)

    clientRef.value = newClient
    await flushPromises()

    expect(chat.currentDialog.value).toBeNull()
    expect(chat.isLoading.value).toBe(false)
  })

  it('loads the dialog lists before the new client confirms the open dialog', async () => {
    const oldClient = createClient({ connected: true, dialogs: [buildDialog('dialog-1')] })
    const newClient = createClient({ connected: true, dialogs: [buildDialog('dialog-new')] })
    vi.mocked(newClient.api.getDialog).mockReturnValue(new Promise(() => {}))
    const clientRef = shallowRef<MTChatClient | null>(oldClient)
    const { chat } = await mountChat({ client: clientRef })
    await chat.selectDialog('dialog-1')

    clientRef.value = newClient
    await flushPromises()

    expect(chat.participatingDialogs.value.map((d) => d.id)).toEqual(['dialog-new'])
    expect(newClient.api.getMessages).not.toHaveBeenCalled()
  })

  it('loads nothing for the open dialog once unmounted', async () => {
    const oldClient = createClient({ connected: true, dialogs: [buildDialog('dialog-1')] })
    const newClient = createClient({ connected: true, dialogs: [buildDialog('dialog-1')] })
    const check = deferred<DialogListItem>()
    vi.mocked(newClient.api.getDialog).mockReturnValue(check.promise)
    const clientRef = shallowRef<MTChatClient | null>(oldClient)
    const { chat, wrapper } = await mountChat({ client: clientRef })
    await chat.selectDialog('dialog-1')
    clientRef.value = newClient
    await flushPromises()

    wrapper.unmount()
    check.resolve(buildDialog('dialog-1'))
    await flushPromises()

    expect(newClient.api.getMessages).not.toHaveBeenCalled()
    expect(newClient.api.getParticipants).not.toHaveBeenCalled()
  })

  it.each([
    ['the client', 'the object'],
    ['the object', 'the client'],
  ])('loads the lists of the new object once when %s changes before %s', async (first) => {
    const oldClient = createClient({ connected: true })
    const newClient = createClient({ connected: true })
    const clientRef = shallowRef<MTChatClient | null>(oldClient)
    const objectIdRef = ref<string | undefined>('a')
    await mountChat({ client: clientRef, objectType: 'order', objectId: objectIdRef })
    vi.mocked(oldClient.api.getDialogsByObject).mockClear()

    if (first === 'the client') {
      clientRef.value = newClient
      objectIdRef.value = 'b'
    } else {
      objectIdRef.value = 'b'
      clientRef.value = newClient
    }
    await flushPromises()

    expect(newClient.api.getDialogsByObject).toHaveBeenCalledTimes(1)
    expect(newClient.api.getDialogsByObject).toHaveBeenCalledWith(
      expect.objectContaining({ objectId: 'b', archived: false })
    )
    expect(oldClient.api.getDialogsByObject).not.toHaveBeenCalled()
  })
  it('does not reload the archive of an object that changed while there was no client', async () => {
    const oldClient = createClient({ connected: true })
    const newClient = createClient({ connected: true })
    const clientRef = shallowRef<MTChatClient | null>(oldClient)
    const objectIdRef = ref<string | undefined>('a')
    const { chat } = await mountChat({ client: clientRef, objectType: 'order', objectId: objectIdRef })
    await chat.loadArchivedDialogsByObject('order', 'a')
    chat.setSearchQuery('query')

    clientRef.value = null
    await flushPromises()
    objectIdRef.value = 'b'
    await flushPromises()
    clientRef.value = newClient
    await flushPromises()

    expect(chat.searchQuery.value).toBe('')
    expect(newClient.api.getDialogsByObject).toHaveBeenCalledWith(
      expect.objectContaining({ objectId: 'b', archived: false, search: undefined })
    )
    expect(newClient.api.getDialogsByObject).not.toHaveBeenCalledWith(expect.objectContaining({ archived: true }))
  })
})

describe('useChat without a client', () => {
  function trackUnhandledRejections(): unknown[] {
    const rejections: unknown[] = []
    const onRejection = (reason: unknown) => rejections.push(reason)
    process.on('unhandledRejection', onRejection)
    onTestFinished(() => { process.off('unhandledRejection', onRejection) })
    return rejections
  }

  it('ignores an object change until a client is set', async () => {
    const rejections = trackUnhandledRejections()
    const objectIdRef = ref<string | undefined>('a')
    const { chat } = await mountChat({ client: shallowRef(null), objectType: 'order', objectId: objectIdRef })

    objectIdRef.value = 'b'
    await flushPromises()
    await new Promise((resolve) => setTimeout(resolve))

    expect(chat.error.value).toBeNull()
    expect(rejections).toEqual([])
  })

  it('loads the archive opened while there was no client once a client is set', async () => {
    const rejections = trackUnhandledRejections()
    const client = createClient({ connected: true })
    vi.mocked(client.api.getParticipatingDialogs).mockImplementation(async (_search, archived) =>
      archived ? [buildDialog('dialog-archived')] : []
    )
    const clientRef = shallowRef<MTChatClient | null>(null)
    const { chat } = await mountChat({ client: clientRef })

    chat.loadArchivedDialogs()
    await flushPromises()
    await new Promise((resolve) => setTimeout(resolve))

    expect(chat.error.value).toBeNull()
    expect(rejections).toEqual([])

    clientRef.value = client
    await flushPromises()

    expect(chat.archivedDialogs.value.map((d) => d.id)).toEqual(['dialog-archived'])
  })

  it('ignores a dialog change until a client is set', async () => {
    const rejections = trackUnhandledRejections()
    const dialogIdRef = ref<string | undefined>()
    const { chat } = await mountChat({ client: shallowRef(null), dialogId: dialogIdRef })

    dialogIdRef.value = 'dialog-1'
    await flushPromises()
    await new Promise((resolve) => setTimeout(resolve))

    expect(chat.error.value).toBeNull()
    expect(rejections).toEqual([])
  })
})

describe('useChat in inline mode', () => {
  it('loads messages and participants once when reloading after a reconnect', async () => {
    const client = createClient({ connected: true })
    vi.spyOn(client.api, 'getDialogByObject').mockResolvedValue(buildDialog('dialog-1'))
    const { chat } = await mountChat({ client: shallowRef(client), mode: 'inline', objectType: 'order', objectId: 'object-1' })
    expect(chat.currentDialog.value?.id).toBe('dialog-1')
    vi.mocked(client.api.getMessages).mockClear()
    vi.mocked(client.api.getParticipants).mockClear()

    emitEvent(client, { type: 'connected' })
    await flushPromises()

    expect(client.api.getMessages).toHaveBeenCalledTimes(1)
    expect(client.api.getParticipants).toHaveBeenCalledTimes(1)
  })
})

describe('useChat in inline mode after a client swap', () => {
  async function mountInlineChatOnDialogA(clientRef: Ref<MTChatClient | null>): Promise<UseChatReturn> {
    const { chat } = await mountChat({ client: clientRef, mode: 'inline', objectType: 'order', objectId: 'object-1' })
    expect(chat.messages.value.map((m) => m.id)).toEqual(['message-a'])
    return chat
  }

  function createClientOnDialogA(): MTChatClient {
    const client = createClient({ connected: true })
    vi.spyOn(client.api, 'getDialogByObject').mockResolvedValue(buildDialog('dialog-a'))
    vi.mocked(client.api.getMessages).mockResolvedValue({ messages: [buildMessage('message-a', 'dialog-a')] })
    vi.mocked(client.api.getParticipants).mockResolvedValue([{ user_id: 'user-a' } as never])
    return client
  }

  it('opens the dialog the new client finds by the object', async () => {
    const newClient = createClient({ connected: true })
    vi.spyOn(newClient.api, 'getDialogByObject').mockResolvedValue(buildDialog('dialog-b'))
    vi.mocked(newClient.api.getMessages).mockResolvedValue({ messages: [buildMessage('message-b', 'dialog-b')] })
    const clientRef = shallowRef<MTChatClient | null>(createClientOnDialogA())
    const chat = await mountInlineChatOnDialogA(clientRef)

    clientRef.value = newClient
    await flushPromises()

    expect(chat.currentDialog.value?.id).toBe('dialog-b')
    expect(chat.messages.value.map((m) => m.id)).toEqual(['message-b'])
    expect(newClient.api.getParticipants).toHaveBeenCalledWith('dialog-b', expect.anything())
    expect(newClient.api.getDialog).not.toHaveBeenCalled()
  })

  it('drops the open dialog when the new client finds none for the object', async () => {
    const newClient = createClient({ connected: true })
    vi.spyOn(newClient.api, 'getDialogByObject').mockResolvedValue(null)
    const clientRef = shallowRef<MTChatClient | null>(createClientOnDialogA())
    const chat = await mountInlineChatOnDialogA(clientRef)

    clientRef.value = newClient
    await flushPromises()

    expect(chat.currentDialog.value).toBeNull()
    expect(chat.messages.value).toEqual([])
    expect(chat.participants.value).toEqual([])
    expect(chat.error.value).toBeNull()
  })

  it.each([
    ['without a client swap', false],
    ['with a client swap in the same tick', true],
  ])('drops the dialog and messages of the old object when the new object has no dialog %s', async (_, isClientSwapped) => {
    const client = createClientOnDialogA()
    const clientRef = shallowRef<MTChatClient | null>(client)
    const objectId = ref<string | undefined>('object-1')
    const { chat } = await mountChat({ client: clientRef, mode: 'inline', objectType: 'order', objectId })
    expect(chat.messages.value.map((m) => m.id)).toEqual(['message-a'])
    const nextClient = isClientSwapped ? createClient({ connected: true }) : client
    vi.spyOn(nextClient.api, 'getDialogByObject').mockResolvedValue(null)

    objectId.value = 'object-2'
    clientRef.value = nextClient
    await flushPromises()

    expect(chat.currentDialog.value).toBeNull()
    expect(chat.messages.value).toEqual([])
    expect(chat.participants.value).toEqual([])
  })
})

describe('useChat archive events', () => {
  it('does not reload lists when the server echoes a manual archive and unarchive', async () => {
    const client = createClient({ connected: true, dialogs: [buildDialog('dialog-1')] })
    const { chat } = await mountChat({ client: shallowRef(client) })
    vi.mocked(client.api.getParticipatingDialogs).mockClear()

    await chat.archiveDialog('dialog-1')
    emitEvent(client, { type: 'dialog.archived', payload: { dialog_id: 'dialog-1' } })

    expect(chat.archivedDialogs.value.map((d) => d.id)).toEqual(['dialog-1'])
    expect(chat.participatingDialogs.value).toEqual([])

    await chat.unarchiveDialog('dialog-1')
    emitEvent(client, { type: 'dialog.unarchived', payload: { dialog_id: 'dialog-1' } })
    await flushPromises()

    expect(chat.participatingDialogs.value.map((d) => d.id)).toEqual(['dialog-1'])
    expect(chat.archivedDialogs.value).toEqual([])
    expect(client.api.getParticipatingDialogs).not.toHaveBeenCalled()
  })

  it('reloads the lists of the object when an unknown dialog is archived', async () => {
    const client = createClient({ connected: true })
    const { chat } = await mountChat({ client: shallowRef(client), objectType: 'order', objectId: 'object-1' })
    vi.mocked(client.api.getDialogsByObject).mockClear()

    emitEvent(client, { type: 'dialog.archived', payload: { dialog_id: 'dialog-unknown' } })
    await flushPromises()

    expect(client.api.getDialogsByObject).toHaveBeenCalledWith(
      expect.objectContaining({ objectType: 'order', objectId: 'object-1', archived: false })
    )
    expect(client.api.getDialogsByObject).toHaveBeenCalledWith(
      expect.objectContaining({ objectType: 'order', objectId: 'object-1', archived: true })
    )
    expect(client.api.getParticipatingDialogs).not.toHaveBeenCalled()
    expect(chat.error.value).toBeNull()
  })
})

describe('useChat with config', () => {
  function stubOwnClient(): MTChatClient[] {
    const clients: MTChatClient[] = []
    vi.spyOn(MTChatClient.prototype, 'connect').mockImplementation(function (this: MTChatClient) {
      clients.push(this)
    })
    vi.spyOn(MTChatClient.prototype, 'disconnect').mockImplementation(() => {})
    vi.spyOn(MTChatClient.prototype, 'isConnected', 'get').mockReturnValue(true)
    vi.spyOn(MTChatApi.prototype, 'getParticipatingDialogs').mockResolvedValue([])
    vi.spyOn(MTChatApi.prototype, 'getAvailableDialogs').mockResolvedValue([])
    return clients
  }

  it('connects on mount and disconnects on unmount', async () => {
    const clients = stubOwnClient()

    const { wrapper } = await mountChat({ config: ref(config) })

    expect(clients).toHaveLength(1)
    wrapper.unmount()
    expect(MTChatClient.prototype.disconnect).toHaveBeenCalledTimes(1)
  })

  it('subscribes to the selected dialog and unsubscribes from the one it leaves', async () => {
    stubOwnClient()
    const subscribe = vi.spyOn(MTChatClient.prototype, 'subscribe').mockImplementation(() => {})
    const unsubscribe = vi.spyOn(MTChatClient.prototype, 'unsubscribe').mockImplementation(() => {})
    vi.spyOn(MTChatApi.prototype, 'getDialog').mockImplementation(async (id) => buildDialog(id))
    vi.spyOn(MTChatApi.prototype, 'getMessages').mockResolvedValue({ messages: [] })
    vi.spyOn(MTChatApi.prototype, 'getParticipants').mockResolvedValue([])
    const { chat, wrapper } = await mountChat({ config: ref(config) })

    await chat.selectDialog('dialog-1')
    expect(subscribe).toHaveBeenCalledWith('dialog-1')

    await chat.selectDialog('dialog-2')
    expect(unsubscribe).toHaveBeenCalledWith('dialog-1')
    expect(subscribe).toHaveBeenCalledWith('dialog-2')

    wrapper.unmount()
    expect(unsubscribe).toHaveBeenCalledWith('dialog-2')
  })

  it('switches to a new client with the new token and ignores the old one', async () => {
    const clients = stubOwnClient()
    const configRef = ref<MTChatConfig>({ ...config, token: 'token-1' })
    const { chat } = await mountChat({ config: configRef })

    configRef.value = { ...config, token: 'token-2' }
    await flushPromises()

    expect(clients).toHaveLength(2)
    const [oldClient, newClient] = clients
    expect(newClient.config.token).toBe('token-2')
    expect(MTChatClient.prototype.disconnect).toHaveBeenCalledTimes(1)

    emitEvent(oldClient, { type: 'connected' })
    expect(chat.isConnected.value).toBe(false)
    emitEvent(newClient, { type: 'connected' })
    expect(chat.isConnected.value).toBe(true)
  })
})
