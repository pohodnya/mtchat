/**
 * Injection Keys for primitive components
 */

import type { InjectionKey } from 'vue'

/**
 * Injection key for MtTabs context
 */
export const TABS_KEY: InjectionKey<{
  activeValue: () => string
  setActiveValue: (value: string) => void
}> = Symbol('mt-tabs')

/**
 * Injection key for MtAccordion context
 */
export const ACCORDION_KEY: InjectionKey<{
  isExpanded: (value: string) => boolean
  toggle: (value: string) => void
}> = Symbol('mt-accordion')
