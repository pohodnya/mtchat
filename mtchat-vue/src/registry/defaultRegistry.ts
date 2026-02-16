/**
 * Default Component Registry
 *
 * Uses native HTML primitives as the default implementation.
 * This ensures MTChat works out of the box without any framework dependencies.
 */

import type { ComponentRegistry } from './types'
import {
  MtButton,
  MtDialog,
  MtMenu,
  MtContextMenu,
  MtInput,
  MtCheckbox,
  MtRadioButton,
  MtTabs,
  MtTab,
  MtAccordion,
  MtAccordionPanel,
  vTooltip,
} from '../primitives'

/**
 * Default registry with native HTML components
 */
export const defaultRegistry: ComponentRegistry = {
  MtButton,
  MtDialog,
  MtMenu,
  MtContextMenu,
  MtInput,
  MtCheckbox,
  MtRadioButton,
  MtTabs,
  MtTab,
  MtAccordion,
  MtAccordionPanel,
  vTooltip,
}
