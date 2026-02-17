import { describe, it, expect } from 'vitest'
import { getInitials, truncateText, getSenderDisplayName } from './helpers'
import type { DialogParticipant } from '../types'

describe('getInitials', () => {
  it('returns first letters of two words', () => {
    expect(getInitials('John Doe')).toBe('JD')
  })

  it('returns first two characters for single word', () => {
    expect(getInitials('Admin')).toBe('AD')
  })

  it('returns "?" for empty string', () => {
    expect(getInitials('')).toBe('?')
  })

  it('handles multiple words (uses first two)', () => {
    expect(getInitials('John Michael Doe')).toBe('JM')
  })

  it('uppercases the result', () => {
    expect(getInitials('john doe')).toBe('JD')
  })

  it('handles single character name', () => {
    const result = getInitials('A')
    expect(result.length).toBeGreaterThan(0)
  })

  it('handles extra whitespace', () => {
    expect(getInitials('  John   Doe  ')).toBe('JD')
  })
})

describe('truncateText', () => {
  it('returns text unchanged if shorter than max', () => {
    expect(truncateText('Hello', 10)).toBe('Hello')
  })

  it('returns text unchanged if exactly max length', () => {
    expect(truncateText('Hello', 5)).toBe('Hello')
  })

  it('truncates and adds ellipsis', () => {
    expect(truncateText('Hello World', 5)).toBe('Hello...')
  })

  it('handles empty string', () => {
    expect(truncateText('', 10)).toBe('')
  })

  it('handles max length of 0', () => {
    expect(truncateText('Hello', 0)).toBe('...')
  })
})

describe('getSenderDisplayName', () => {
  const participants: DialogParticipant[] = [
    {
      dialog_id: 'dialog-1',
      user_id: 'user-1',
      display_name: 'John Doe',
      company: 'Acme',
      joined_at: '2024-01-01',
      joined_as: 'participant',
      notifications_enabled: true,
      unread_count: 0,
    },
    {
      dialog_id: 'dialog-1',
      user_id: 'user-2',
      display_name: 'Jane Smith',
      company: 'Corp',
      joined_at: '2024-01-01',
      joined_as: 'participant',
      notifications_enabled: true,
      unread_count: 0,
    },
  ]

  it('returns display name when found', () => {
    expect(getSenderDisplayName('user-1', participants, 'user-3', 'You')).toBe('John Doe')
  })

  it('returns "You" label for current user not in participants', () => {
    expect(getSenderDisplayName('user-3', participants, 'user-3', 'You')).toBe('You')
  })

  it('returns truncated ID for unknown user', () => {
    const result = getSenderDisplayName('unknown-user-id', participants, 'user-3', 'You')
    expect(result).toBe('unknown-')
    expect(result.length).toBe(8)
  })

  it('returns display name even for current user if in participants', () => {
    expect(getSenderDisplayName('user-1', participants, 'user-1', 'You')).toBe('John Doe')
  })

  it('handles empty participants array', () => {
    // Falls back to truncated ID (first 8 chars)
    expect(getSenderDisplayName('user-1-long-id', [], 'user-2', 'You')).toBe('user-1-l')
  })
})
