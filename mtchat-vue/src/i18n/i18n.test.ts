import { describe, it, expect } from 'vitest'
import { translations, type Locale } from './translations'

/**
 * Test the translation interpolation logic directly.
 * We replicate the `tt` logic here since createI18nHelpers is not exported.
 */
function createTestTt(locale: Locale) {
  const t = translations[locale] || translations.ru

  return function tt(key: string, params?: Record<string, string | number>): string {
    const keys = key.split('.')
    let value: unknown = t

    for (const k of keys) {
      if (value && typeof value === 'object' && k in value) {
        value = (value as Record<string, unknown>)[k]
      } else {
        return key
      }
    }

    if (typeof value !== 'string') {
      return key
    }

    if (params) {
      return value.replace(/\{(\w+)\}/g, (_, paramKey) => {
        return params[paramKey] !== undefined ? String(params[paramKey]) : `{${paramKey}}`
      })
    }

    return value
  }
}

describe('translations', () => {
  it('has all three locales', () => {
    expect(translations).toHaveProperty('ru')
    expect(translations).toHaveProperty('en')
    expect(translations).toHaveProperty('zh')
  })

  it('has consistent keys across locales', () => {
    const ruKeys = Object.keys(translations.ru)
    const enKeys = Object.keys(translations.en)
    const zhKeys = Object.keys(translations.zh)

    expect(enKeys).toEqual(expect.arrayContaining(ruKeys))
    expect(zhKeys).toEqual(expect.arrayContaining(ruKeys))
  })

  it('has required top-level sections', () => {
    for (const locale of ['ru', 'en', 'zh'] as Locale[]) {
      const t = translations[locale]
      expect(t).toHaveProperty('tabs')
      expect(t).toHaveProperty('buttons')
      expect(t).toHaveProperty('chat')
      expect(t).toHaveProperty('status')
      expect(t).toHaveProperty('dates')
    }
  })
})

describe('tt (translation interpolation)', () => {
  const tt = createTestTt('en')

  it('resolves simple key', () => {
    expect(tt('status.connected')).toBe('Connected')
    expect(tt('status.disconnected')).toBe('Disconnected')
  })

  it('resolves nested key', () => {
    expect(tt('tabs.myChats')).toBeTruthy()
    expect(tt('tabs.available')).toBeTruthy()
  })

  it('returns key for missing translation', () => {
    expect(tt('nonexistent.key')).toBe('nonexistent.key')
  })

  it('interpolates parameters', () => {
    const result = tt('chat.participants', { count: 5 })
    expect(result).toContain('5')
  })

  it('preserves unreplaced placeholders', () => {
    const result = tt('chat.participants', {})
    expect(result).toContain('{count}')
  })

  it('returns key for non-string value', () => {
    // 'tabs' is an object, not a string
    expect(tt('tabs')).toBe('tabs')
  })
})

describe('tt for Russian locale', () => {
  const tt = createTestTt('ru')

  it('resolves Russian translations', () => {
    expect(tt('status.connected')).toBe('Подключено')
    expect(tt('status.disconnected')).toBe('Отключено')
  })

  it('interpolates with Russian text', () => {
    const result = tt('chat.participants', { count: 3 })
    expect(result).toContain('3')
  })
})

describe('tt for Chinese locale', () => {
  const tt = createTestTt('zh')

  it('resolves Chinese translations', () => {
    expect(tt('status.connected')).toBeTruthy()
    expect(tt('status.disconnected')).toBeTruthy()
  })
})
