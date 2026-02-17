<script setup lang="ts">
/**
 * MTChatPrime - MTChat with PrimeVue UI components and theme integration
 *
 * This component wraps MTChat and automatically configures:
 * - PrimeVue component registry (buttons, dialogs, inputs, etc.)
 * - Theme token mapping (uses --p-* CSS variables from your PrimeVue preset)
 *
 * Theme modes:
 * - 'light' (default): uses light surface tokens
 * - 'dark': uses dark surface tokens
 * - undefined: auto-detects from PrimeVue dark mode selector (.dark-mode, .p-dark)
 *
 * Customization:
 * - Override PrimeVue tokens (--p-*) via your preset configuration
 * - Override MTChat tokens (--mtchat-*) via CSS on .mtchat-prime class
 */

import { computed } from 'vue'
import { MTChat, provideRegistry, type MTChatProps } from '@mtchat/vue'
import { primevueRegistry } from '../registry/primevueRegistry'
import '../theme/aura.css'

const props = withDefaults(
  defineProps<Omit<MTChatProps, 'theme'> & {
    /** Theme: 'light', 'dark', or undefined for auto-detect via PrimeVue dark mode */
    theme?: 'light' | 'dark'
  }>(),
  {
    mode: 'full',
    showHeader: true,
    showSidebar: true,
    theme: 'light',
  }
)

const wrapperClass = computed(() => [
  'mtchat-prime',
  props.theme ? `mtchat-prime--${props.theme}` : null,
])

defineEmits<{
  (e: 'connected'): void
  (e: 'disconnected'): void
  (e: 'error', error: Error): void
  (e: 'message-sent', message: unknown): void
  (e: 'header-menu-action', dialog: unknown): void
}>()

// Provide PrimeVue registry to all child components
provideRegistry(primevueRegistry)
</script>

<template>
  <div :class="wrapperClass">
    <MTChat
      :config="config"
      :mode="mode"
      :object-id="objectId"
      :object-type="objectType"
      :dialog-id="dialogId"
      :show-header="showHeader"
      :show-sidebar="showSidebar"
      :theme="theme"
      @connected="$emit('connected')"
      @disconnected="$emit('disconnected')"
      @error="$emit('error', $event)"
      @message-sent="$emit('message-sent', $event)"
      @header-menu-action="$emit('header-menu-action', $event)"
    >
      <!-- Pass through named slots -->
      <template v-if="$slots['sidebar-action']" #sidebar-action>
        <slot name="sidebar-action" />
      </template>
      <template v-if="$slots['header-menu-actions']" #header-menu-actions="scope">
        <slot name="header-menu-actions" v-bind="scope" />
      </template>
    </MTChat>
  </div>
</template>

<style scoped>
.mtchat-prime {
  height: 100%;
  width: 100%;
}
</style>
