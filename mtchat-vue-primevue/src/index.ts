/**
 * MTChat Vue PrimeVue Integration
 *
 * Provides PrimeVue components for MTChat UI primitives.
 *
 * @example Basic usage with provideRegistry
 * ```vue
 * <script setup>
 * import { MTChat, provideRegistry } from '@mtchat/vue'
 * import { primevueRegistry } from '@mtchat/vue-primevue'
 *
 * // Use PrimeVue components instead of native HTML
 * provideRegistry(primevueRegistry)
 * </script>
 * <template>
 *   <MTChat :config="config" />
 * </template>
 * ```
 *
 * @example Ready-to-use MTChatPrime component
 * ```vue
 * <script setup>
 * import { MTChatPrime } from '@mtchat/vue-primevue'
 * </script>
 * <template>
 *   <MTChatPrime :config="config" />
 * </template>
 * ```
 */

// Ready-to-use component with PrimeVue registry pre-configured
export { default as MTChatPrime } from './components/MTChatPrime.vue'

// Registry
export { primevueRegistry } from './registry/primevueRegistry'

// Individual PrimeVue adapter components (for custom registry building)
export {
  PrimeButton,
  PrimeDialog,
  PrimeMenu,
  PrimeContextMenu,
  PrimeInput,
  PrimeCheckbox,
  PrimeRadioButton,
  PrimeTabs,
  PrimeTab,
  PrimeAccordion,
  PrimeAccordionPanel,
} from './primitives'

// Re-export types from @mtchat/vue for convenience
export type {
  ComponentRegistry,
  PartialRegistry,
  MtButtonProps,
  MtDialogProps,
  MtMenuProps,
  MtMenuItem,
  MtInputProps,
  MtCheckboxProps,
  MtRadioButtonProps,
  MtTabsProps,
  MtTabProps,
  MtAccordionProps,
  MtAccordionPanelProps,
} from '@mtchat/vue'
