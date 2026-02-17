import { describe, it, expect } from 'vitest'
import {
  getAttachmentType,
  isAllowedFileType,
  isValidFileSize,
  formatFileSize,
  getFileIconName,
  ATTACHMENT_LIMITS,
} from './index'

describe('getAttachmentType', () => {
  it('returns "image" for image types', () => {
    expect(getAttachmentType('image/jpeg')).toBe('image')
    expect(getAttachmentType('image/png')).toBe('image')
    expect(getAttachmentType('image/gif')).toBe('image')
    expect(getAttachmentType('image/webp')).toBe('image')
    expect(getAttachmentType('image/svg+xml')).toBe('image')
  })

  it('returns "pdf" for PDF', () => {
    expect(getAttachmentType('application/pdf')).toBe('pdf')
  })

  it('returns "file" for other types', () => {
    expect(getAttachmentType('application/zip')).toBe('file')
    expect(getAttachmentType('text/plain')).toBe('file')
    expect(getAttachmentType('video/mp4')).toBe('file')
    expect(getAttachmentType('audio/mpeg')).toBe('file')
  })
})

describe('isAllowedFileType', () => {
  it('allows common image types', () => {
    expect(isAllowedFileType('image/jpeg')).toBe(true)
    expect(isAllowedFileType('image/png')).toBe(true)
    expect(isAllowedFileType('image/gif')).toBe(true)
  })

  it('allows document types', () => {
    expect(isAllowedFileType('application/pdf')).toBe(true)
    expect(isAllowedFileType('application/msword')).toBe(true)
    expect(isAllowedFileType('text/plain')).toBe(true)
  })

  it('allows archive types', () => {
    expect(isAllowedFileType('application/zip')).toBe(true)
  })

  it('allows audio/video types', () => {
    expect(isAllowedFileType('audio/mpeg')).toBe(true)
    expect(isAllowedFileType('video/mp4')).toBe(true)
  })

  it('rejects unknown types', () => {
    expect(isAllowedFileType('application/x-executable')).toBe(false)
    expect(isAllowedFileType('application/octet-stream')).toBe(false)
  })

  it('allows empty content type', () => {
    expect(isAllowedFileType('')).toBe(true)
  })
})

describe('isValidFileSize', () => {
  it('accepts valid sizes', () => {
    expect(isValidFileSize(1)).toBe(true)
    expect(isValidFileSize(1024)).toBe(true)
    expect(isValidFileSize(50 * 1024 * 1024)).toBe(true) // 50MB
  })

  it('accepts exactly max size', () => {
    expect(isValidFileSize(ATTACHMENT_LIMITS.MAX_FILE_SIZE)).toBe(true)
  })

  it('rejects zero size', () => {
    expect(isValidFileSize(0)).toBe(false)
  })

  it('rejects negative size', () => {
    expect(isValidFileSize(-1)).toBe(false)
  })

  it('rejects oversized files', () => {
    expect(isValidFileSize(ATTACHMENT_LIMITS.MAX_FILE_SIZE + 1)).toBe(false)
    expect(isValidFileSize(200 * 1024 * 1024)).toBe(false) // 200MB
  })
})

describe('formatFileSize', () => {
  it('formats bytes', () => {
    expect(formatFileSize(500)).toBe('500 B')
    expect(formatFileSize(0)).toBe('0 B')
  })

  it('formats kilobytes', () => {
    expect(formatFileSize(1024)).toBe('1.0 KB')
    expect(formatFileSize(2048)).toBe('2.0 KB')
    expect(formatFileSize(1536)).toBe('1.5 KB')
  })

  it('formats megabytes', () => {
    expect(formatFileSize(1024 * 1024)).toBe('1.0 MB')
    expect(formatFileSize(5 * 1024 * 1024)).toBe('5.0 MB')
    expect(formatFileSize(1.5 * 1024 * 1024)).toBe('1.5 MB')
  })
})

describe('getFileIconName', () => {
  it('returns "file-pdf" for PDF', () => {
    expect(getFileIconName('application/pdf')).toBe('file-pdf')
  })

  it('returns "image" for images', () => {
    expect(getFileIconName('image/jpeg')).toBe('image')
    expect(getFileIconName('image/png')).toBe('image')
  })

  it('returns "file-spreadsheet" for spreadsheets', () => {
    expect(getFileIconName('application/vnd.ms-excel')).toBe('file-spreadsheet')
    expect(getFileIconName('application/vnd.openxmlformats-officedocument.spreadsheetml.sheet')).toBe('file-spreadsheet')
    expect(getFileIconName('text/csv')).toBe('file-spreadsheet')
  })

  it('returns "file-document" for documents', () => {
    expect(getFileIconName('application/msword')).toBe('file-document')
    expect(getFileIconName('application/vnd.openxmlformats-officedocument.wordprocessingml.document')).toBe('file-document')
  })

  it('returns "file-archive" for archives', () => {
    expect(getFileIconName('application/zip')).toBe('file-archive')
    expect(getFileIconName('application/x-rar-compressed')).toBe('file-archive')
  })

  it('returns "file-video" for video', () => {
    expect(getFileIconName('video/mp4')).toBe('file-video')
  })

  it('returns "file-audio" for audio', () => {
    expect(getFileIconName('audio/mpeg')).toBe('file-audio')
  })

  it('returns "file" as default', () => {
    expect(getFileIconName('application/octet-stream')).toBe('file')
    expect(getFileIconName(undefined)).toBe('file')
  })
})
