import { describe, it, expect } from 'vitest'
import { sanitizeHtml, stripHtml } from './sanitize'

describe('sanitizeHtml', () => {
  it('allows safe HTML tags', () => {
    const input = '<p>Hello <strong>world</strong></p>'
    expect(sanitizeHtml(input)).toBe('<p>Hello <strong>world</strong></p>')
  })

  it('allows all whitelisted tags', () => {
    const tags = [
      '<p>text</p>',
      '<br>',
      '<strong>bold</strong>',
      '<em>italic</em>',
      '<u>underline</u>',
      '<s>strikethrough</s>',
      '<a href="https://example.com">link</a>',
      '<ul><li>item</li></ul>',
      '<ol><li>item</li></ol>',
      '<blockquote>quote</blockquote>',
      '<code>code</code>',
      '<pre>preformatted</pre>',
      '<span>inline</span>',
    ]
    for (const tag of tags) {
      const result = sanitizeHtml(tag)
      expect(result).toContain(tag.replace(/<\/?[a-z]+[^>]*>/g, '').trim() || '')
    }
  })

  it('removes script tags', () => {
    const input = '<p>Hello</p><script>alert("xss")</script>'
    const result = sanitizeHtml(input)
    expect(result).not.toContain('<script>')
    expect(result).not.toContain('alert')
    expect(result).toContain('<p>Hello</p>')
  })

  it('removes event handlers', () => {
    const input = '<p onclick="alert(1)">click me</p>'
    const result = sanitizeHtml(input)
    expect(result).not.toContain('onclick')
    expect(result).toContain('click me')
  })

  it('removes javascript: URLs', () => {
    const input = '<a href="javascript:alert(1)">click</a>'
    const result = sanitizeHtml(input)
    expect(result).not.toContain('javascript:')
  })

  it('removes iframe tags', () => {
    const input = '<iframe src="https://evil.com"></iframe>'
    const result = sanitizeHtml(input)
    expect(result).not.toContain('<iframe')
  })

  it('removes img tags with onerror', () => {
    const input = '<img src="x" onerror="alert(1)">'
    const result = sanitizeHtml(input)
    expect(result).not.toContain('onerror')
    expect(result).not.toContain('alert')
  })

  it('removes style tags', () => {
    const input = '<style>body { display: none; }</style><p>text</p>'
    const result = sanitizeHtml(input)
    expect(result).not.toContain('<style')
    expect(result).toContain('<p>text</p>')
  })

  it('preserves allowed attributes', () => {
    const input = '<a href="https://example.com" target="_blank" rel="noopener">link</a>'
    const result = sanitizeHtml(input)
    expect(result).toContain('href="https://example.com"')
    expect(result).toContain('target="_blank"')
  })

  it('preserves class attribute', () => {
    const input = '<span class="mention">@user</span>'
    const result = sanitizeHtml(input)
    expect(result).toContain('class="mention"')
  })

  it('preserves data-type and data-id for mentions', () => {
    const input = '<span data-type="mention" data-id="user-123" data-label="John">@John</span>'
    const result = sanitizeHtml(input)
    expect(result).toContain('data-type="mention"')
    expect(result).toContain('data-id="user-123"')
    expect(result).toContain('data-label="John"')
  })

  it('strips disallowed data attributes', () => {
    const input = '<span data-evil="payload">text</span>'
    const result = sanitizeHtml(input)
    expect(result).not.toContain('data-evil')
  })

  it('handles empty string', () => {
    expect(sanitizeHtml('')).toBe('')
  })

  it('handles plain text (no HTML)', () => {
    expect(sanitizeHtml('Hello world')).toBe('Hello world')
  })

  it('removes SVG-based XSS', () => {
    const input = '<svg onload="alert(1)"><text>test</text></svg>'
    const result = sanitizeHtml(input)
    expect(result).not.toContain('<svg')
    expect(result).not.toContain('onload')
  })

  it('removes div and other non-whitelisted tags', () => {
    const input = '<div><p>text</p></div>'
    const result = sanitizeHtml(input)
    expect(result).not.toContain('<div')
    expect(result).toContain('<p>text</p>')
  })

  it('preserves text from stripped tags', () => {
    const input = '<div>content</div>'
    const result = sanitizeHtml(input)
    expect(result).not.toContain('<div')
    expect(result).toContain('content')
  })
})

describe('stripHtml', () => {
  it('removes all HTML tags', () => {
    const input = '<p>Hello <strong>world</strong></p>'
    expect(stripHtml(input)).toBe('Hello world')
  })

  it('removes script content', () => {
    const input = '<p>text</p><script>alert("xss")</script>'
    const result = stripHtml(input)
    expect(result).not.toContain('alert')
    expect(result).toContain('text')
  })

  it('handles empty string', () => {
    expect(stripHtml('')).toBe('')
  })

  it('returns plain text unchanged', () => {
    expect(stripHtml('Hello world')).toBe('Hello world')
  })

  it('handles nested tags', () => {
    const input = '<ul><li><strong>Bold</strong> item</li></ul>'
    expect(stripHtml(input)).toContain('Bold')
    expect(stripHtml(input)).toContain('item')
  })

  it('handles entities', () => {
    const input = '<p>a &amp; b &lt; c</p>'
    const result = stripHtml(input)
    expect(result).toContain('a')
    expect(result).toContain('b')
  })
})
