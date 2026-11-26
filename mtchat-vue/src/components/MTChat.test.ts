import { describe, it, expect, vi, afterEach } from 'vitest'
import { shallowMount, flushPromises } from '@vue/test-utils'
import MTChat from './MTChat.vue'
import ChatSidebar from './chat/ChatSidebar.vue'
import JoinDialog from './chat/JoinDialog.vue'
import ChatInput from './chat/ChatInput.vue'
import { MTChatClient } from '../sdk/client'
import type { DialogListItem, MTChatConfig } from '../types'

const clientConfig: MTChatConfig = {
  baseUrl: 'https://chat.example.com',
  userId: 'client-user',
  locale: 'en',
  scopeConfig: { scopeLevel0: ['tenant-1'], scopeLevel1: [], scopeLevel2: [] },
  userProfile: {
    displayName: 'Client User',
    company: 'Company A',
    email: 'user@example.com',
    phone: '+10000000000',
  },
}

const dialog: DialogListItem = {
  id: 'dialog-1',
  object_id: 'object-1',
  object_type: 'order',
  created_at: '2026-01-01T00:00:00Z',
  participants_count: 2,
  i_am_participant: true,
}

function createClient(): MTChatClient {
  const client = new MTChatClient(clientConfig)
  vi.spyOn(client, 'connect').mockImplementation(() => {})
  vi.spyOn(client, 'disconnect').mockImplementation(() => {})
  vi.spyOn(client.api, 'getParticipatingDialogs').mockResolvedValue([dialog])
  vi.spyOn(client.api, 'getAvailableDialogs').mockResolvedValue([])
  vi.spyOn(client.api, 'getMessages').mockResolvedValue({ messages: [] })
  vi.spyOn(client.api, 'getParticipants').mockResolvedValue([])
  vi.spyOn(client.api, 'getDialog').mockResolvedValue(dialog)
  vi.spyOn(client.api, 'getPresignedUploadUrl').mockResolvedValue({ upload_url: 'https://s3.example.com/upload', s3_key: 'key', expires_in: 300 })
  vi.spyOn(client.api, 'uploadFile').mockResolvedValue()
  return client
}

afterEach(() => {
  vi.restoreAllMocks()
})

describe('MTChat with client and without config', () => {
  it('takes userId, userProfile and locale from client.config', async () => {
    const client = createClient()

    const wrapper = shallowMount(MTChat, { props: { client } })
    await flushPromises()

    expect(wrapper.findComponent(ChatSidebar).props('currentUserId')).toBe('client-user')
    expect(wrapper.findComponent(JoinDialog).props()).toMatchObject({
      profileName: 'Client User',
      company: 'Company A',
      email: 'user@example.com',
      phone: '+10000000000',
    })
    expect(wrapper.text()).toContain('Select a chat to start messaging')
    expect(client.connect).not.toHaveBeenCalled()

    wrapper.unmount()
    expect(client.disconnect).not.toHaveBeenCalled()
  })
})

describe('MTChat with a replaced client', () => {
  it('uploads a file through the new client', async () => {
    const oldClient = createClient()
    const newClient = createClient()
    const wrapper = shallowMount(MTChat, {
      props: { client: oldClient, dialogId: 'dialog-1' },
      global: { stubs: { ChatInput: false } },
    })
    await flushPromises()

    await wrapper.setProps({ client: newClient })
    await flushPromises()
    const input = wrapper.find('input[type="file"]')
    const file = new File(['%PDF'], 'doc.pdf', { type: 'application/pdf' })
    Object.defineProperty(input.element, 'files', { value: [file] })
    await input.trigger('change')
    await flushPromises()

    expect(newClient.api.getPresignedUploadUrl).toHaveBeenCalledTimes(1)
    expect(oldClient.api.getPresignedUploadUrl).not.toHaveBeenCalled()
  })
})

describe('MTChat without a client', () => {
  it('renders the empty state when neither client nor config is passed', async () => {
    const wrapper = shallowMount(MTChat, { props: { client: undefined } })
    await flushPromises()

    expect(wrapper.text()).toContain('Выберите чат для начала переписки')
    expect(wrapper.findComponent(ChatSidebar).props('currentUserId')).toBe('')
    expect(wrapper.findComponent(ChatInput).exists()).toBe(false)
  })

  it('renders the empty state when the client is removed with a dialog open', async () => {
    const client = createClient()
    const wrapper = shallowMount(MTChat, { props: { client, dialogId: 'dialog-1' } })
    await flushPromises()
    expect(wrapper.findComponent(ChatInput).exists()).toBe(true)

    await wrapper.setProps({ client: null })
    await flushPromises()

    expect(wrapper.text()).toContain('Select a chat to start messaging')
    expect(wrapper.findComponent(ChatInput).exists()).toBe(false)
    expect(wrapper.findComponent(ChatSidebar).props('participatingDialogs')).toEqual([])
  })
})
