/**
 * MTChat i18n Composable
 *
 * Provides translation functions for components.
 * Uses Vue's provide/inject for locale propagation.
 */

import { computed, inject, provide, ref, type ComputedRef, type InjectionKey, type Ref } from 'vue'
import { translations, type Locale, type TranslationStrings } from './translations'

// Injection key for locale
export const I18N_LOCALE_KEY: InjectionKey<Ref<Locale>> = Symbol('mtchat-i18n-locale')

/**
 * Provide i18n locale to child components
 * Call this in the root MTChat component
 * Returns the same interface as useI18n() so the root component can use it directly
 */
export function provideI18n(locale: Locale = 'ru'): {
  t: ComputedRef<TranslationStrings>
  locale: Ref<Locale>
  tt: (key: string, params?: Record<string, string | number>) => string
  formatDate: (date: Date) => string
  formatDateDivider: (dateString: string) => string
  localeRef: Ref<Locale>
} {
  const localeRef = ref(locale)
  provide(I18N_LOCALE_KEY, localeRef)

  // Return the same interface as useI18n for the provider component
  const t = computed(() => translations[localeRef.value] || translations.ru)

  function tt(key: string, params?: Record<string, string | number>): string {
    const keys = key.split('.')
    let value: unknown = t.value

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

  function formatDate(date: Date): string {
    const now = new Date()
    const today = new Date(now.getFullYear(), now.getMonth(), now.getDate())
    const dateOnly = new Date(date.getFullYear(), date.getMonth(), date.getDate())

    if (dateOnly.getTime() === today.getTime()) {
      return t.value.dates.today
    }

    const yesterday = new Date(today)
    yesterday.setDate(yesterday.getDate() - 1)
    if (dateOnly.getTime() === yesterday.getTime()) {
      return t.value.dates.yesterday
    }

    const localeCode = localeRef.value === 'zh' ? 'zh-CN' : localeRef.value
    const options: Intl.DateTimeFormatOptions = {
      day: 'numeric',
      month: 'long',
    }

    if (date.getFullYear() !== now.getFullYear()) {
      options.year = 'numeric'
    }

    return new Intl.DateTimeFormat(localeCode, options).format(date)
  }

  function formatDateDivider(dateString: string): string {
    return formatDate(new Date(dateString))
  }

  return {
    t,
    locale: localeRef,
    tt,
    formatDate,
    formatDateDivider,
    localeRef,
  }
}

/**
 * i18n composable for components
 * Returns translation object and helper functions
 */
export function useI18n(): {
  t: ComputedRef<TranslationStrings>
  locale: Ref<Locale>
  tt: (key: string, params?: Record<string, string | number>) => string
  formatDate: (date: Date) => string
  formatDateDivider: (dateString: string) => string
} {
  // Inject locale from parent, default to 'ru'
  const locale = inject(I18N_LOCALE_KEY, ref('ru' as Locale))

  // Get translation object for current locale
  const t = computed(() => translations[locale.value] || translations.ru)

  /**
   * Translate with interpolation
   * Usage: tt('chat.participants', { count: 5 })
   * Returns: "5 участников"
   */
  function tt(key: string, params?: Record<string, string | number>): string {
    const keys = key.split('.')
    let value: unknown = t.value

    for (const k of keys) {
      if (value && typeof value === 'object' && k in value) {
        value = (value as Record<string, unknown>)[k]
      } else {
        return key // Key not found, return as-is
      }
    }

    if (typeof value !== 'string') {
      return key
    }

    // Replace placeholders: {name} -> value
    if (params) {
      return value.replace(/\{(\w+)\}/g, (_, paramKey) => {
        return params[paramKey] !== undefined ? String(params[paramKey]) : `{${paramKey}}`
      })
    }

    return value
  }

  /**
   * Format date for display (used in date dividers)
   */
  function formatDate(date: Date): string {
    const now = new Date()
    const today = new Date(now.getFullYear(), now.getMonth(), now.getDate())
    const dateOnly = new Date(date.getFullYear(), date.getMonth(), date.getDate())

    // Check if today
    if (dateOnly.getTime() === today.getTime()) {
      return t.value.dates.today
    }

    // Check if yesterday
    const yesterday = new Date(today)
    yesterday.setDate(yesterday.getDate() - 1)
    if (dateOnly.getTime() === yesterday.getTime()) {
      return t.value.dates.yesterday
    }

    // Format with Intl.DateTimeFormat for locale-aware formatting
    const localeCode = locale.value === 'zh' ? 'zh-CN' : locale.value
    const options: Intl.DateTimeFormatOptions = {
      day: 'numeric',
      month: 'long',
    }

    // Add year if not current year
    if (date.getFullYear() !== now.getFullYear()) {
      options.year = 'numeric'
    }

    return new Intl.DateTimeFormat(localeCode, options).format(date)
  }

  /**
   * Format date divider from ISO string
   */
  function formatDateDivider(dateString: string): string {
    return formatDate(new Date(dateString))
  }

  return {
    t,
    locale,
    tt,
    formatDate,
    formatDateDivider,
  }
}
