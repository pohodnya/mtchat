<template>
  <div id="app" class="tms-mode">
    <router-view />
  </div>
</template>

<script setup lang="ts">
// All pages now use TMSLayout directly
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

#app.tms-mode {
  background: #1a1a2e;
}

/* Global dark theme overrides for PrimeVue overlays/dropdowns */

/* Select dropdown panel */
.p-select-overlay {
  background: #1e2a4a !important;
  border: 1px solid rgba(255, 255, 255, 0.15) !important;
}

.p-select-overlay .p-select-option {
  color: #e0e0e0 !important;
}

.p-select-overlay .p-select-option:hover {
  background: rgba(79, 195, 247, 0.15) !important;
}

.p-select-overlay .p-select-option.p-highlight {
  background: rgba(79, 195, 247, 0.25) !important;
  color: #4fc3f7 !important;
}

.p-select-overlay .p-select-option .p-select-option-check-icon {
  color: #4fc3f7 !important;
}

/* MultiSelect dropdown panel */
.p-multiselect-overlay {
  background: #1e2a4a !important;
  border: 1px solid rgba(255, 255, 255, 0.15) !important;
}

.p-multiselect-overlay .p-multiselect-option {
  color: #e0e0e0 !important;
}

.p-multiselect-overlay .p-multiselect-option:hover {
  background: rgba(79, 195, 247, 0.15) !important;
}

.p-multiselect-overlay .p-multiselect-option.p-highlight {
  background: rgba(79, 195, 247, 0.25) !important;
}

.p-multiselect-header {
  background: #1e2a4a !important;
  border-color: rgba(255, 255, 255, 0.1) !important;
  color: #e0e0e0 !important;
}

/* Confirm Dialog */
.p-confirmdialog {
  background: #1e2a4a !important;
  border: 1px solid rgba(255, 255, 255, 0.15) !important;
}

.p-dialog-header {
  background: #1e2a4a !important;
  color: #e0e0e0 !important;
  border-color: rgba(255, 255, 255, 0.1) !important;
}

.p-dialog-content {
  background: #1e2a4a !important;
  color: #b0b0b0 !important;
}

.p-dialog-footer {
  background: #1e2a4a !important;
  border-color: rgba(255, 255, 255, 0.1) !important;
}

/* Toast */
.p-toast .p-toast-message {
  background: #1e2a4a !important;
  border: 1px solid rgba(255, 255, 255, 0.15) !important;
}

.p-toast .p-toast-message-content {
  color: #e0e0e0 !important;
}

.p-toast .p-toast-summary {
  color: #fff !important;
}

.p-toast .p-toast-detail {
  color: #b0b0b0 !important;
}

/* Tooltip */
.p-tooltip .p-tooltip-text {
  background: #2a3a5a !important;
  color: #e0e0e0 !important;
}

/* TabView global overrides */
.p-tabview {
  background: transparent !important;
}

.p-tabview-tablist,
.p-tabview-nav,
.p-tabview-nav-container,
.p-tabview-nav-content,
.p-tabview-tab-list {
  background: transparent !important;
}

.p-tabview-panels,
.p-tabview-panel {
  background: transparent !important;
}

.p-tabview-header,
.p-tab {
  background: transparent !important;
}

.p-tabview-header a,
.p-tabview-nav-link,
.p-tab {
  background: transparent !important;
  color: #888 !important;
}

.p-tabview-header a:hover,
.p-tabview-nav-link:hover,
.p-tab:hover {
  background: rgba(255, 255, 255, 0.05) !important;
  color: #e0e0e0 !important;
}

.p-tabview-header.p-highlight a,
.p-tabview-header.p-tabview-selected a,
.p-tabview-nav-link[data-p-active="true"],
.p-tab[data-p-active="true"],
.p-tab.p-tab-active {
  color: #4fc3f7 !important;
  background: transparent !important;
}

.p-tabview-ink-bar {
  background: #4fc3f7 !important;
}

/* Chips/InputChips component (PrimeVue 4) */
.p-chips,
.p-inputchips {
  background: rgba(255, 255, 255, 0.08) !important;
  border-color: rgba(255, 255, 255, 0.15) !important;
}

.p-inputchips-input {
  background: rgba(255, 255, 255, 0.08) !important;
  border-color: rgba(255, 255, 255, 0.15) !important;
  color: #e0e0e0 !important;
}

.p-inputchips-input::placeholder {
  color: #666 !important;
}

.p-inputchips-input-item input {
  background: transparent !important;
  color: #e0e0e0 !important;
}

.p-inputchips-input-item input::placeholder {
  color: #666 !important;
}

.p-inputchips-chip,
.p-inputchips-chip-item .p-chip,
.p-chips-token,
.p-chip {
  background: #4fc3f7 !important;
  color: #1a1a2e !important;
}

.p-inputchips-chip-icon,
.p-chips-token-icon,
.p-chip-remove-icon {
  color: #1a1a2e !important;
}

/* Legacy Chips classes */
.p-chips-multiple-container,
.p-chips-input-token {
  background: rgba(255, 255, 255, 0.08) !important;
  border-color: rgba(255, 255, 255, 0.15) !important;
}

.p-chips-multiple-container input,
.p-chips-input-token input {
  background: transparent !important;
  color: #e0e0e0 !important;
}

.p-chips-multiple-container input::placeholder,
.p-chips-input-token input::placeholder {
  color: #666 !important;
}
</style>
