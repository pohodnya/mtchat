/**
 * Component Registry Types
 *
 * Defines interfaces for all UI primitive components that can be swapped
 * between native HTML implementations and framework-specific ones (e.g., PrimeVue)
 */

import type { Component } from 'vue'

// ============ Button ============

export type MtButtonVariant = 'primary' | 'secondary' | 'danger' | 'ghost' | 'text'
export type MtButtonSize = 'sm' | 'md' | 'lg'

export interface MtButtonProps {
  /** Button variant style */
  variant?: MtButtonVariant
  /** Button size */
  size?: MtButtonSize
  /** Disabled state */
  disabled?: boolean
  /** Loading state (shows spinner) */
  loading?: boolean
  /** Button type attribute */
  type?: 'button' | 'submit' | 'reset'
  /** Icon-only button (square padding) */
  icon?: boolean
  /** Title/tooltip text */
  title?: string
}

// ============ Dialog ============

export interface MtDialogProps {
  /** Whether dialog is visible */
  visible: boolean
  /** Dialog title */
  header?: string
  /** Whether clicking outside closes dialog */
  modal?: boolean
  /** Whether to show close button */
  closable?: boolean
  /** Max width CSS value */
  maxWidth?: string
  /** Whether dialog can be dragged */
  draggable?: boolean
  /** Theme name */
  theme?: string
}

export interface MtDialogEmits {
  (e: 'update:visible', value: boolean): void
  (e: 'close'): void
}

// ============ Menu ============

export interface MtMenuItem {
  /** Menu item label */
  label: string
  /** Icon name (optional) */
  icon?: string
  /** Click handler */
  command?: () => void
  /** Whether item is disabled */
  disabled?: boolean
  /** Whether item is a separator */
  separator?: boolean
  /** Danger style (red text) */
  danger?: boolean
}

export interface MtMenuProps {
  /** Menu items */
  items: MtMenuItem[]
  /** Popup mode (positioned near trigger) */
  popup?: boolean
}

export interface MtMenuEmits {
  (e: 'select', item: MtMenuItem): void
  (e: 'hide'): void
}

export interface MtMenuExpose {
  /** Toggle menu visibility */
  toggle: (event: Event) => void
  /** Show menu */
  show: (event: Event) => void
  /** Hide menu */
  hide: () => void
}

// ============ Context Menu ============

export interface MtContextMenuProps {
  /** Menu items */
  items: MtMenuItem[]
}

export interface MtContextMenuEmits {
  (e: 'select', item: MtMenuItem): void
  (e: 'hide'): void
}

export interface MtContextMenuExpose {
  /** Show context menu at position */
  show: (event: MouseEvent) => void
  /** Hide context menu */
  hide: () => void
}

// ============ Input ============

export interface MtInputProps {
  /** Input value (v-model) */
  modelValue: string
  /** Placeholder text */
  placeholder?: string
  /** Input type */
  type?: 'text' | 'password' | 'email' | 'url' | 'search'
  /** Disabled state */
  disabled?: boolean
  /** Invalid state */
  invalid?: boolean
  /** Clearable (show clear button) */
  clearable?: boolean
  /** Size */
  size?: 'sm' | 'md' | 'lg'
}

export interface MtInputEmits {
  (e: 'update:modelValue', value: string): void
  (e: 'clear'): void
  (e: 'focus', event: FocusEvent): void
  (e: 'blur', event: FocusEvent): void
  (e: 'keydown', event: KeyboardEvent): void
}

export interface MtInputExpose {
  /** Focus the input */
  focus: () => void
  /** Select input text */
  select: () => void
}

// ============ Checkbox ============

export interface MtCheckboxProps {
  /** Checked state (v-model) */
  modelValue: boolean
  /** Label text */
  label?: string
  /** Disabled state */
  disabled?: boolean
  /** Input name */
  name?: string
}

export interface MtCheckboxEmits {
  (e: 'update:modelValue', value: boolean): void
}

// ============ Radio Button ============

export interface MtRadioButtonProps {
  /** Selected value (v-model) */
  modelValue: unknown
  /** Radio button value */
  value: unknown
  /** Label text */
  label?: string
  /** Disabled state */
  disabled?: boolean
  /** Input name (for grouping) */
  name?: string
}

export interface MtRadioButtonEmits {
  (e: 'update:modelValue', value: unknown): void
}

// ============ Tabs ============

export interface MtTabsProps {
  /** Active tab value (v-model) */
  modelValue: string
}

export interface MtTabsEmits {
  (e: 'update:modelValue', value: string): void
}

export interface MtTabProps {
  /** Tab value */
  value: string
  /** Tab label */
  label: string
  /** Badge value (e.g., count) */
  badge?: string | number
  /** Disabled state */
  disabled?: boolean
}

// ============ Accordion ============

export interface MtAccordionProps {
  /** Active panel values (v-model) */
  modelValue?: string | string[]
  /** Allow multiple panels open */
  multiple?: boolean
}

export interface MtAccordionEmits {
  (e: 'update:modelValue', value: string | string[]): void
}

export interface MtAccordionPanelProps {
  /** Panel value (unique identifier) */
  value: string
  /** Panel header text */
  header: string
  /** Disabled state */
  disabled?: boolean
}

// ============ Tooltip ============

export interface MtTooltipOptions {
  /** Tooltip text */
  value?: string
  /** Position */
  position?: 'top' | 'bottom' | 'left' | 'right'
  /** Delay before showing (ms) */
  showDelay?: number
  /** Delay before hiding (ms) */
  hideDelay?: number
}

// ============ Component Registry ============

/**
 * Registry of UI primitive components
 * All components should follow Vue 3 component interface
 */
export interface ComponentRegistry {
  // Components
  MtButton: Component
  MtDialog: Component
  MtMenu: Component
  MtContextMenu: Component
  MtInput: Component
  MtCheckbox: Component
  MtRadioButton: Component
  MtTabs: Component
  MtTab: Component
  MtAccordion: Component
  MtAccordionPanel: Component

  // Directives (use unknown to allow different directive implementations)
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  vTooltip: unknown
}

/**
 * Partial registry for extending/overriding specific components
 */
export type PartialRegistry = Partial<ComponentRegistry>
