<script setup lang="ts">
/**
 * MTChatPrime - MTChat with PrimeVue UI components
 *
 * This component wraps MTChat and automatically configures the PrimeVue registry.
 * Use this for a ready-to-use chat with PrimeVue styling.
 */

import { MTChat, provideRegistry, type MTChatProps } from '@mtchat/vue'
import { primevueRegistry } from '../registry/primevueRegistry'

defineProps<MTChatProps>()

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
</template>
