import type { DialogParticipant } from '../types'

/**
 * Get initials from a display name (first letters of first two words).
 */
export function getInitials(name: string): string {
  if (!name) return '?'
  const parts = name.trim().split(/\s+/)
  if (parts.length >= 2) {
    return (parts[0][0] + parts[1][0]).toUpperCase()
  }
  return name.slice(0, 2).toUpperCase()
}

/**
 * Truncate text to a maximum length, adding ellipsis if needed.
 */
export function truncateText(text: string, maxLength: number): string {
  if (text.length <= maxLength) return text
  return text.slice(0, maxLength) + '...'
}

/**
 * Get a participant's display name, falling back to "You" label or truncated ID.
 */
export function getSenderDisplayName(
  senderId: string,
  participants: DialogParticipant[],
  currentUserId: string,
  youLabel: string,
): string {
  const participant = participants.find(p => p.user_id === senderId)
  if (participant?.display_name) return participant.display_name
  return senderId === currentUserId ? youLabel : senderId.slice(0, 8)
}

/**
 * Whether a rejection is a request we cancelled ourselves.
 *
 * Aborting a fetch rejects it like any other failure, so every catch that
 * handles request errors has to tell the two apart: a cancelled request is not
 * a problem to report, retry, or use to conclude anything about the dialog
 * (e.g. "there are no more messages").
 *
 * Browsers reject with a DOMException named 'AbortError'; happy-dom and older
 * polyfills use a plain Error with the same name, hence the duck-typing.
 */
export function isAbortError(e: unknown): boolean {
  return e instanceof Error && e.name === 'AbortError'
}
