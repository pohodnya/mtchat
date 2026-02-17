/**
 * HTML sanitization utilities using DOMPurify
 *
 * Provides client-side sanitization as defense-in-depth
 * (backend also sanitizes with ammonia crate)
 */

import DOMPurify from 'dompurify'

/**
 * Allowed HTML tags matching the backend ammonia configuration:
 * p, br, strong, em, u, s, a, ul, ol, li, blockquote, code, pre, span
 */
const ALLOWED_TAGS = [
  'p', 'br', 'strong', 'em', 'u', 's', 'a',
  'ul', 'ol', 'li', 'blockquote', 'code', 'pre', 'span',
]

const ALLOWED_ATTR = ['href', 'target', 'rel', 'class', 'data-type', 'data-id', 'data-label']

/**
 * Sanitize HTML content for safe rendering via v-html.
 * Only allows tags matching the backend ammonia whitelist.
 */
export function sanitizeHtml(html: string): string {
  return DOMPurify.sanitize(html, {
    ALLOWED_TAGS,
    ALLOWED_ATTR,
    ALLOW_DATA_ATTR: false,
  })
}

/**
 * Strip all HTML tags and return plain text.
 * First sanitizes with DOMPurify to remove dangerous content (scripts, etc.),
 * then extracts text content via DOM parsing.
 */
export function stripHtml(html: string): string {
  if (!html) return ''
  const clean = DOMPurify.sanitize(html)
  const doc = new DOMParser().parseFromString(clean, 'text/html')
  return doc.body.textContent || ''
}
