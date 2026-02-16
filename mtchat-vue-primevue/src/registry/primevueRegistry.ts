/**
 * PrimeVue Component Registry
 *
 * Provides PrimeVue implementations for all MTChat UI primitives.
 */

import type { ComponentRegistry } from '@mtchat/vue'
import Tooltip from 'primevue/tooltip'
import {
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
} from '../primitives'

/**
 * PrimeVue registry with all components mapped to PrimeVue equivalents
 */
export const primevueRegistry: ComponentRegistry = {
  MtButton: PrimeButton,
  MtDialog: PrimeDialog,
  MtMenu: PrimeMenu,
  MtContextMenu: PrimeContextMenu,
  MtInput: PrimeInput,
  MtCheckbox: PrimeCheckbox,
  MtRadioButton: PrimeRadioButton,
  MtTabs: PrimeTabs,
  MtTab: PrimeTab,
  MtAccordion: PrimeAccordion,
  MtAccordionPanel: PrimeAccordionPanel,
  vTooltip: Tooltip,
}
