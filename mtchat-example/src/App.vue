<template>
  <div id="app" :class="`theme-${theme}`">
    <router-view />
    <Toast position="top-right" />
  </div>
</template>

<script setup lang="ts">
import { computed, watch } from 'vue'
import { useToast } from 'primevue/usetoast'
import Toast from 'primevue/toast'
import { useSettings, useWebhookNotifications, useUsers } from './composables'

const { settings } = useSettings()
const theme = computed(() => settings.value.theme)

// Get current user for filtering notifications
const { currentUser } = useUsers()

// Webhook notifications
const toast = useToast()
const { lastEvent, isConnected } = useWebhookNotifications()

// Show toast when webhook arrives
watch(lastEvent, (event) => {
  if (!event) return

  console.log('[Webhook] Processing event:', event.type, 'recipient:', event.payload?.recipient_id, 'current user:', currentUser.value?.id)

  if (event.type === 'notification_pending') {
    // Smart notification - message not read after delay
    // Only show for current user
    const recipientId = event.payload?.recipient_id
    if (!currentUser.value || recipientId !== currentUser.value.id) {
      return
    }

    const message = event.payload?.message

    // Extract text content from HTML
    const content = message?.content?.replace(/<[^>]*>/g, '') || ''
    const truncated = content.length > 100 ? content.slice(0, 100) + '...' : content

    toast.add({
      severity: 'info',
      summary: 'New unread message',
      detail: truncated,
      life: 10000,
    })
  } else if (event.type === 'message_new') {
    // Instant message webhook - don't show toast (use notification_pending for smart notifications)
  } else if (event.type === 'participant_joined' || event.type === 'participant_left') {
    // Only show for current user's dialogs (skip for now - no user filtering available)
  }
})

// Log connection status
watch(isConnected, (connected) => {
  if (connected) {
    console.log('[App] Webhook SSE connected')
  }
})
</script>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
}

#app {
  min-height: 100vh;
}

/* ========== DARK THEME PrimeVue Overrides ========== */

.theme-dark .p-select-overlay {
  background: #111827 !important;
  border: 1px solid #374151 !important;
}

.theme-dark .p-select-overlay .p-select-option {
  color: #f8fafc !important;
}

.theme-dark .p-select-overlay .p-select-option:hover {
  background: rgba(96, 165, 250, 0.15) !important;
}

.theme-dark .p-select-overlay .p-select-option.p-highlight {
  background: rgba(96, 165, 250, 0.25) !important;
  color: #60a5fa !important;
}

.theme-dark .p-select-overlay .p-select-option .p-select-option-check-icon {
  color: #60a5fa !important;
}

.theme-dark .p-multiselect-overlay {
  background: #111827 !important;
  border: 1px solid #374151 !important;
}

.theme-dark .p-multiselect-overlay .p-multiselect-option {
  color: #f8fafc !important;
}

.theme-dark .p-multiselect-overlay .p-multiselect-option:hover {
  background: rgba(96, 165, 250, 0.15) !important;
}

.theme-dark .p-multiselect-overlay .p-multiselect-option.p-highlight {
  background: rgba(96, 165, 250, 0.25) !important;
}

.theme-dark .p-multiselect-header {
  background: #111827 !important;
  border-color: #374151 !important;
  color: #f8fafc !important;
}

.theme-dark .p-confirmdialog,
.theme-dark .p-dialog {
  background: #111827 !important;
  border: 1px solid #374151 !important;
}

.theme-dark .p-dialog-header {
  background: #111827 !important;
  color: #f8fafc !important;
  border-color: #374151 !important;
}

.theme-dark .p-dialog-content {
  background: #111827 !important;
  color: #94a3b8 !important;
}

.theme-dark .p-dialog-footer {
  background: #111827 !important;
  border-color: #374151 !important;
}

.theme-dark .p-toast .p-toast-message {
  background: #111827 !important;
  border: 1px solid #374151 !important;
}

.theme-dark .p-toast .p-toast-message-content {
  color: #f8fafc !important;
}

.theme-dark .p-toast .p-toast-summary {
  color: #fff !important;
}

.theme-dark .p-toast .p-toast-detail {
  color: #94a3b8 !important;
}

.theme-dark .p-tooltip .p-tooltip-text {
  background: #1f2937 !important;
  color: #f8fafc !important;
}

.theme-dark .p-tabview {
  background: transparent !important;
}

.theme-dark .p-tabview-tablist,
.theme-dark .p-tabview-nav,
.theme-dark .p-tabview-nav-container,
.theme-dark .p-tabview-nav-content,
.theme-dark .p-tabview-tab-list {
  background: transparent !important;
}

.theme-dark .p-tabview-panels,
.theme-dark .p-tabview-panel {
  background: transparent !important;
}

.theme-dark .p-tabview-header,
.theme-dark .p-tab {
  background: transparent !important;
}

.theme-dark .p-tabview-header a,
.theme-dark .p-tabview-nav-link,
.theme-dark .p-tab {
  background: transparent !important;
  color: #94a3b8 !important;
}

.theme-dark .p-tabview-header a:hover,
.theme-dark .p-tabview-nav-link:hover,
.theme-dark .p-tab:hover {
  background: #374151 !important;
  color: #f8fafc !important;
}

.theme-dark .p-tabview-header.p-highlight a,
.theme-dark .p-tabview-header.p-tabview-selected a,
.theme-dark .p-tabview-nav-link[data-p-active="true"],
.theme-dark .p-tab[data-p-active="true"],
.theme-dark .p-tab.p-tab-active {
  color: #60a5fa !important;
  background: transparent !important;
}

.theme-dark .p-tabview-ink-bar {
  background: #60a5fa !important;
}

.theme-dark .p-chips,
.theme-dark .p-inputchips {
  background: #1f2937 !important;
  border-color: #374151 !important;
}

.theme-dark .p-inputchips-input {
  background: #1f2937 !important;
  border-color: #374151 !important;
  color: #f8fafc !important;
}

.theme-dark .p-inputchips-input::placeholder {
  color: #64748b !important;
}

.theme-dark .p-inputchips-input-item input {
  background: transparent !important;
  color: #f8fafc !important;
}

.theme-dark .p-inputchips-input-item input::placeholder {
  color: #64748b !important;
}

.theme-dark .p-inputchips-chip,
.theme-dark .p-inputchips-chip-item .p-chip,
.theme-dark .p-chips-token,
.theme-dark .p-chip {
  background: #60a5fa !important;
  color: #1f2937 !important;
}

.theme-dark .p-inputchips-chip-icon,
.theme-dark .p-chips-token-icon,
.theme-dark .p-chip-remove-icon {
  color: #1f2937 !important;
}

.theme-dark .p-chips-multiple-container,
.theme-dark .p-chips-input-token {
  background: #1f2937 !important;
  border-color: #374151 !important;
}

.theme-dark .p-chips-multiple-container input,
.theme-dark .p-chips-input-token input {
  background: transparent !important;
  color: #f8fafc !important;
}

.theme-dark .p-chips-multiple-container input::placeholder,
.theme-dark .p-chips-input-token input::placeholder {
  color: #64748b !important;
}

/* ========== LIGHT THEME PrimeVue Overrides ========== */

.theme-light .p-select-overlay {
  background: #ffffff !important;
  border: 1px solid #e2e8f0 !important;
}

.theme-light .p-select-overlay .p-select-option {
  color: #334155 !important;
}

.theme-light .p-select-overlay .p-select-option:hover {
  background: rgba(59, 130, 246, 0.1) !important;
}

.theme-light .p-select-overlay .p-select-option.p-highlight {
  background: rgba(59, 130, 246, 0.15) !important;
  color: #3B82F6 !important;
}

.theme-light .p-select-overlay .p-select-option .p-select-option-check-icon {
  color: #3B82F6 !important;
}

.theme-light .p-multiselect-overlay {
  background: #ffffff !important;
  border: 1px solid #e2e8f0 !important;
}

.theme-light .p-multiselect-overlay .p-multiselect-option {
  color: #334155 !important;
}

.theme-light .p-multiselect-overlay .p-multiselect-option:hover {
  background: rgba(59, 130, 246, 0.1) !important;
}

.theme-light .p-multiselect-overlay .p-multiselect-option.p-highlight {
  background: rgba(59, 130, 246, 0.15) !important;
}

.theme-light .p-multiselect-header {
  background: #ffffff !important;
  border-color: #e2e8f0 !important;
  color: #334155 !important;
}

.theme-light .p-confirmdialog,
.theme-light .p-dialog {
  background: #ffffff !important;
  border: 1px solid #e2e8f0 !important;
}

.theme-light .p-dialog-header {
  background: #ffffff !important;
  color: #334155 !important;
  border-color: #e2e8f0 !important;
}

.theme-light .p-dialog-content {
  background: #ffffff !important;
  color: #64748b !important;
}

.theme-light .p-dialog-footer {
  background: #ffffff !important;
  border-color: #e2e8f0 !important;
}

.theme-light .p-toast .p-toast-message {
  background: #ffffff !important;
  border: 1px solid #e2e8f0 !important;
}

.theme-light .p-toast .p-toast-message-content {
  color: #334155 !important;
}

.theme-light .p-toast .p-toast-summary {
  color: #1e293b !important;
}

.theme-light .p-toast .p-toast-detail {
  color: #64748b !important;
}

.theme-light .p-tooltip .p-tooltip-text {
  background: #f8fafc !important;
  color: #334155 !important;
  border: 1px solid #e2e8f0;
}

.theme-light .p-tabview {
  background: transparent !important;
}

.theme-light .p-tabview-tablist,
.theme-light .p-tabview-nav,
.theme-light .p-tabview-nav-container,
.theme-light .p-tabview-nav-content,
.theme-light .p-tabview-tab-list {
  background: transparent !important;
}

.theme-light .p-tabview-panels,
.theme-light .p-tabview-panel {
  background: transparent !important;
}

.theme-light .p-tabview-header,
.theme-light .p-tab {
  background: transparent !important;
}

.theme-light .p-tabview-header a,
.theme-light .p-tabview-nav-link,
.theme-light .p-tab {
  background: transparent !important;
  color: #64748b !important;
}

.theme-light .p-tabview-header a:hover,
.theme-light .p-tabview-nav-link:hover,
.theme-light .p-tab:hover {
  background: #f1f5f9 !important;
  color: #334155 !important;
}

.theme-light .p-tabview-header.p-highlight a,
.theme-light .p-tabview-header.p-tabview-selected a,
.theme-light .p-tabview-nav-link[data-p-active="true"],
.theme-light .p-tab[data-p-active="true"],
.theme-light .p-tab.p-tab-active {
  color: #3B82F6 !important;
  background: transparent !important;
}

.theme-light .p-tabview-ink-bar {
  background: #3B82F6 !important;
}

.theme-light .p-chips,
.theme-light .p-inputchips {
  background: #ffffff !important;
  border-color: #e2e8f0 !important;
}

.theme-light .p-inputchips-input {
  background: #ffffff !important;
  border-color: #e2e8f0 !important;
  color: #334155 !important;
}

.theme-light .p-inputchips-input::placeholder {
  color: #94a3b8 !important;
}

.theme-light .p-inputchips-input-item input {
  background: transparent !important;
  color: #334155 !important;
}

.theme-light .p-inputchips-input-item input::placeholder {
  color: #94a3b8 !important;
}

.theme-light .p-inputchips-chip,
.theme-light .p-inputchips-chip-item .p-chip,
.theme-light .p-chips-token,
.theme-light .p-chip {
  background: #3B82F6 !important;
  color: #ffffff !important;
}

.theme-light .p-inputchips-chip-icon,
.theme-light .p-chips-token-icon,
.theme-light .p-chip-remove-icon {
  color: #ffffff !important;
}

.theme-light .p-chips-multiple-container,
.theme-light .p-chips-input-token {
  background: #ffffff !important;
  border-color: #e2e8f0 !important;
}

.theme-light .p-chips-multiple-container input,
.theme-light .p-chips-input-token input {
  background: transparent !important;
  color: #334155 !important;
}

.theme-light .p-chips-multiple-container input::placeholder,
.theme-light .p-chips-input-token input::placeholder {
  color: #94a3b8 !important;
}

/* DataTable overrides for light theme */
.theme-light .p-datatable {
  background: transparent !important;
}

.theme-light .p-datatable-table {
  background: transparent !important;
}

.theme-light .p-datatable-thead > tr > th {
  background: #f8fafc !important;
  color: #1e293b !important;
  border-color: #e2e8f0 !important;
  font-weight: 600 !important;
}

.theme-light .p-datatable-tbody > tr {
  background: #ffffff !important;
  color: #334155 !important;
}

.theme-light .p-datatable-tbody > tr > td {
  border-color: #e2e8f0 !important;
  color: #334155 !important;
}

.theme-light .p-datatable-tbody > tr:nth-child(even) {
  background: #f8fafc !important;
}

.theme-light .p-datatable-tbody > tr:hover {
  background: #f1f5f9 !important;
}

.theme-light .p-datatable-emptymessage {
  color: #64748b !important;
}

.theme-light .p-paginator {
  background: transparent !important;
  border: none !important;
}

.theme-light .p-paginator .p-paginator-page,
.theme-light .p-paginator .p-paginator-next,
.theme-light .p-paginator .p-paginator-prev,
.theme-light .p-paginator .p-paginator-first,
.theme-light .p-paginator .p-paginator-last {
  color: #64748b !important;
}

.theme-light .p-paginator .p-paginator-page.p-highlight {
  background: #3B82F6 !important;
  color: #ffffff !important;
}

/* InputText and form controls for light theme */
.theme-light .p-inputtext {
  background: #ffffff !important;
  border-color: #e2e8f0 !important;
  color: #334155 !important;
}

.theme-light .p-inputtext::placeholder {
  color: #94a3b8 !important;
}

.theme-light .p-inputtext:focus {
  border-color: #3B82F6 !important;
  box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.2) !important;
}

.theme-light .p-select {
  background: #ffffff !important;
  border-color: #e2e8f0 !important;
  color: #334155 !important;
}

.theme-light .p-select:focus,
.theme-light .p-select.p-focus {
  border-color: #3B82F6 !important;
  box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.2) !important;
}

.theme-light .p-select-label {
  color: #334155 !important;
}

.theme-light .p-select-label.p-placeholder {
  color: #94a3b8 !important;
}

.theme-light .p-multiselect {
  background: #ffffff !important;
  border-color: #e2e8f0 !important;
  color: #334155 !important;
}

.theme-light .p-multiselect:focus,
.theme-light .p-multiselect.p-focus {
  border-color: #3B82F6 !important;
  box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.2) !important;
}

.theme-light .p-multiselect-label {
  color: #334155 !important;
}

.theme-light .p-multiselect-label.p-placeholder {
  color: #94a3b8 !important;
}

.theme-light .p-textarea {
  background: #ffffff !important;
  border-color: #e2e8f0 !important;
  color: #334155 !important;
}

.theme-light .p-textarea::placeholder {
  color: #94a3b8 !important;
}

.theme-light .p-textarea:focus {
  border-color: #3B82F6 !important;
  box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.2) !important;
}
</style>
