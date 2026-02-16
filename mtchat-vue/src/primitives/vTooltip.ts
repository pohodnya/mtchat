/**
 * vTooltip - Native HTML tooltip directive
 *
 * Uses title attribute as fallback with optional enhanced behavior
 */

import type { Directive, DirectiveBinding } from 'vue'
import type { MtTooltipOptions } from '../registry/types'

export interface TooltipElement extends HTMLElement {
  _tooltipText?: string
  _tooltipCleanup?: () => void
}

function getTooltipText(binding: DirectiveBinding<string | MtTooltipOptions>): string {
  if (typeof binding.value === 'string') {
    return binding.value
  }
  return binding.value?.value || ''
}

function setupTooltip(el: TooltipElement, binding: DirectiveBinding<string | MtTooltipOptions>) {
  const text = getTooltipText(binding)
  el._tooltipText = text

  // Simple implementation: just use title attribute
  // This provides native browser tooltip behavior
  if (text) {
    el.setAttribute('title', text)
  } else {
    el.removeAttribute('title')
  }
}

function cleanup(el: TooltipElement) {
  el.removeAttribute('title')
  if (el._tooltipCleanup) {
    el._tooltipCleanup()
    el._tooltipCleanup = undefined
  }
}

export const vTooltip: Directive<TooltipElement, string | MtTooltipOptions> = {
  mounted(el, binding) {
    setupTooltip(el, binding)
  },

  updated(el, binding) {
    setupTooltip(el, binding)
  },

  unmounted(el) {
    cleanup(el)
  },
}

export default vTooltip
