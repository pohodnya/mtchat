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
