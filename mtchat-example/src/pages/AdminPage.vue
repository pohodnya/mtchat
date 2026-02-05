<template>
  <TMSLayout :show-banner="false">
    <div class="admin-content">
      <!-- Page Header -->
      <div class="page-header">
        <div class="page-title">
          <i class="pi pi-cog" />
          <span>Admin Panel</span>
        </div>
        <p class="page-subtitle">Manage tenants, users, objects, and dialogs for demo</p>
      </div>

      <!-- Tabs -->
      <div class="admin-tabs">
        <button
          v-for="tab in tabs"
          :key="tab.key"
          class="tab-btn"
          :class="{ active: activeTab === tab.key }"
          @click="activeTab = tab.key"
        >
          <i :class="tab.icon" />
          {{ tab.label }}
        </button>
      </div>

      <!-- Tab Content -->
      <div class="tab-content">
        <TenantsPanel v-if="activeTab === 'tenants'" />
        <UsersPanel v-else-if="activeTab === 'users'" />
        <ObjectsPanel v-else-if="activeTab === 'objects'" />
        <DialogsPanel v-else-if="activeTab === 'dialogs'" />
        <SettingsPanel v-else-if="activeTab === 'settings'" />
      </div>
    </div>
  </TMSLayout>
  <Toast />
</template>

<script setup lang="ts">
import { ref } from 'vue'
import Toast from 'primevue/toast'
import TMSLayout from '../components/TMSLayout.vue'
import TenantsPanel from '../components/admin/TenantsPanel.vue'
import UsersPanel from '../components/admin/UsersPanel.vue'
import ObjectsPanel from '../components/admin/ObjectsPanel.vue'
import DialogsPanel from '../components/admin/DialogsPanel.vue'
import SettingsPanel from '../components/admin/SettingsPanel.vue'

const activeTab = ref('tenants')

const tabs = [
  { key: 'tenants', label: 'Tenants', icon: 'pi pi-building' },
  { key: 'users', label: 'Users', icon: 'pi pi-users' },
  { key: 'objects', label: 'Objects', icon: 'pi pi-box' },
  { key: 'dialogs', label: 'Dialogs', icon: 'pi pi-comments' },
  { key: 'settings', label: 'Settings', icon: 'pi pi-cog' },
]
</script>

<style scoped>
.admin-content {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

/* Page Header */
.page-header {
  padding: 20px 24px;
  border-bottom: 1px solid var(--tms-border, rgba(255, 255, 255, 0.1));
}

.page-title {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 18px;
  font-weight: 600;
  color: var(--tms-text, #fff);
}

.page-title i {
  color: var(--tms-primary, #4fc3f7);
}

.page-subtitle {
  margin-top: 4px;
  font-size: 14px;
  color: var(--tms-text-secondary, #888);
}

/* Tabs */
.admin-tabs {
  display: flex;
  gap: 0;
  padding: 0 24px;
  border-bottom: 1px solid var(--tms-border, rgba(255, 255, 255, 0.1));
  background: var(--tms-bg-hover, rgba(255, 255, 255, 0.02));
}

.tab-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 14px 20px;
  background: none;
  border: none;
  color: var(--tms-text-secondary, #888);
  font-size: 14px;
  cursor: pointer;
  border-bottom: 2px solid transparent;
  margin-bottom: -1px;
  transition: all 0.2s;
}

.tab-btn:hover {
  color: var(--tms-text, #e0e0e0);
  background: var(--tms-bg-hover, rgba(255, 255, 255, 0.03));
}

.tab-btn.active {
  color: var(--tms-primary, #4fc3f7);
  border-bottom-color: var(--tms-primary, #4fc3f7);
}

.tab-btn i {
  font-size: 14px;
}

/* Tab Content */
.tab-content {
  flex: 1;
  overflow: auto;
  padding: 24px;
}

/* Override panel styles for dark theme */
.tab-content :deep(.panel) {
  background: transparent !important;
}

.tab-content :deep(.form-section),
.tab-content :deep(.list-section) {
  background: rgba(255, 255, 255, 0.05) !important;
  border: 1px solid rgba(255, 255, 255, 0.1) !important;
  box-shadow: none !important;
}

.tab-content :deep(h3) {
  color: #b0b0b0 !important;
}

.tab-content :deep(label) {
  color: #b0b0b0 !important;
}

.tab-content :deep(.form-field label) {
  color: #888 !important;
}

/* DataTable */
.tab-content :deep(.p-datatable) {
  background: transparent !important;
}

.tab-content :deep(.p-datatable-table) {
  background: transparent !important;
}

.tab-content :deep(.p-datatable .p-datatable-thead > tr > th) {
  background: rgba(255, 255, 255, 0.05) !important;
  color: #b0b0b0 !important;
  border-color: rgba(255, 255, 255, 0.1) !important;
}

.tab-content :deep(.p-datatable .p-datatable-tbody > tr) {
  background: transparent !important;
  color: #e0e0e0 !important;
}

.tab-content :deep(.p-datatable .p-datatable-tbody > tr > td) {
  border-color: rgba(255, 255, 255, 0.05) !important;
}

.tab-content :deep(.p-datatable .p-datatable-tbody > tr:hover) {
  background: rgba(255, 255, 255, 0.05) !important;
}

.tab-content :deep(.p-datatable .p-datatable-tbody > tr.p-datatable-row-odd) {
  background: rgba(255, 255, 255, 0.02) !important;
}

.tab-content :deep(.p-datatable-empty-message) {
  color: #888 !important;
}

/* Form inputs */
.tab-content :deep(.p-inputtext),
.tab-content :deep(.p-select),
.tab-content :deep(.p-multiselect),
.tab-content :deep(.p-chips),
.tab-content :deep(.p-textarea) {
  background: rgba(255, 255, 255, 0.08) !important;
  border-color: rgba(255, 255, 255, 0.15) !important;
  color: #e0e0e0 !important;
}

.tab-content :deep(.p-inputtext::placeholder),
.tab-content :deep(.p-textarea::placeholder) {
  color: #666 !important;
}

.tab-content :deep(.p-select-label) {
  color: #e0e0e0 !important;
}

.tab-content :deep(.p-select-dropdown) {
  color: #888 !important;
}

.tab-content :deep(.p-chips-input-token input) {
  color: #e0e0e0 !important;
}

.tab-content :deep(.p-chips-input-token input::placeholder) {
  color: #666 !important;
}

.tab-content :deep(.p-chips-token) {
  background: #4fc3f7 !important;
  color: #1a1a2e !important;
}

/* Buttons */
.tab-content :deep(.p-button) {
  background: #4fc3f7;
  border-color: #4fc3f7;
  color: #1a1a2e;
}

.tab-content :deep(.p-button:hover) {
  background: #29b6f6;
  border-color: #29b6f6;
}

.tab-content :deep(.p-button.p-button-text) {
  background: transparent;
  color: #4fc3f7;
}

.tab-content :deep(.p-button.p-button-danger.p-button-text) {
  color: #ef5350;
}

/* Tags */
.tab-content :deep(.p-tag) {
  font-size: 11px;
}

.tab-content :deep(.p-tag.p-tag-info) {
  background: rgba(79, 195, 247, 0.2);
  color: #4fc3f7;
}

.tab-content :deep(.p-tag.p-tag-success) {
  background: rgba(102, 187, 106, 0.2);
  color: #66bb6a;
}

/* Code/UUID */
.tab-content :deep(.uuid),
.tab-content :deep(code) {
  background: rgba(255, 255, 255, 0.1) !important;
  color: #b0b0b0 !important;
}

/* Message component */
.tab-content :deep(.p-message) {
  background: rgba(255, 193, 7, 0.15) !important;
  border-color: rgba(255, 193, 7, 0.3) !important;
}

.tab-content :deep(.p-message .p-message-text) {
  color: #ffc107 !important;
}

/* Dialog info boxes */
.tab-content :deep(.dialog-info),
.tab-content :deep(.no-chat-info) {
  background: rgba(255, 255, 255, 0.05) !important;
  border: 1px solid rgba(255, 255, 255, 0.1) !important;
}

.tab-content :deep(.dialog-info h4) {
  color: #888 !important;
}

.tab-content :deep(.dialog-info dt) {
  color: #666 !important;
}

.tab-content :deep(.dialog-info dd) {
  color: #e0e0e0 !important;
}

/* Scopes list */
.tab-content :deep(.scopes-list) {
  color: #e0e0e0;
}

.tab-content :deep(.scope-row) {
  color: #e0e0e0;
}

/* Empty states */
.tab-content :deep(.empty-scope) {
  color: #666 !important;
}

/* Pagination */
.tab-content :deep(.p-paginator) {
  background: transparent !important;
  border: none !important;
  color: #888 !important;
}

.tab-content :deep(.p-paginator .p-paginator-page),
.tab-content :deep(.p-paginator .p-paginator-first),
.tab-content :deep(.p-paginator .p-paginator-prev),
.tab-content :deep(.p-paginator .p-paginator-next),
.tab-content :deep(.p-paginator .p-paginator-last) {
  color: #888 !important;
  background: transparent !important;
}

.tab-content :deep(.p-paginator .p-paginator-page.p-highlight) {
  background: rgba(79, 195, 247, 0.2) !important;
  color: #4fc3f7 !important;
}

/* PrimeVue TabView inside panels */
.tab-content :deep(.p-tabview) {
  background: transparent !important;
}

.tab-content :deep(.p-tabview-panels) {
  background: transparent !important;
  padding: 16px 0 0 !important;
}

.tab-content :deep(.p-tabview-panel) {
  background: transparent !important;
}

.tab-content :deep(.p-tabview-tablist) {
  background: transparent !important;
  border-color: rgba(255, 255, 255, 0.1) !important;
}

.tab-content :deep(.p-tabview-nav) {
  background: transparent !important;
  border-color: rgba(255, 255, 255, 0.1) !important;
}

.tab-content :deep(.p-tabview-nav-container) {
  background: transparent !important;
}

.tab-content :deep(.p-tabview-nav-content) {
  background: transparent !important;
}

.tab-content :deep(.p-tabview-tab-list) {
  background: transparent !important;
}

.tab-content :deep(.p-tabview-header) {
  background: transparent !important;
}

.tab-content :deep(.p-tabview-header a),
.tab-content :deep(.p-tabview-header .p-tabview-nav-link) {
  background: transparent !important;
  color: #888 !important;
  border-color: transparent !important;
}

.tab-content :deep(.p-tabview-header a:hover),
.tab-content :deep(.p-tabview-header .p-tabview-nav-link:hover) {
  color: #e0e0e0 !important;
  background: rgba(255, 255, 255, 0.05) !important;
}

.tab-content :deep(.p-tabview-ink-bar) {
  background: #4fc3f7 !important;
}

.tab-content :deep(.p-tabview-nav-link[data-p-active="true"]),
.tab-content :deep(.p-tabview-header.p-tabview-selected a),
.tab-content :deep(.p-tabview-header.p-tabview-selected .p-tabview-nav-link),
.tab-content :deep(.p-tabview-header.p-highlight a),
.tab-content :deep(.p-tabview-header.p-highlight .p-tabview-nav-link) {
  color: #4fc3f7 !important;
  border-color: #4fc3f7 !important;
  background: transparent !important;
}

.tab-content :deep(.p-tab) {
  background: transparent !important;
  color: #888 !important;
}

.tab-content :deep(.p-tab:hover) {
  color: #e0e0e0 !important;
  background: rgba(255, 255, 255, 0.05) !important;
}

.tab-content :deep(.p-tab[data-p-active="true"]),
.tab-content :deep(.p-tab.p-tab-active) {
  color: #4fc3f7 !important;
  background: transparent !important;
}

/* Description text */
.tab-content :deep(.description) {
  color: #888 !important;
}

/* Empty state */
.tab-content :deep(.empty-state) {
  color: #666 !important;
}

.tab-content :deep(.empty-state i) {
  color: #444 !important;
}

/* Action cards (Settings panel) */
.tab-content :deep(.action-card) {
  background: rgba(255, 255, 255, 0.03) !important;
  border-color: rgba(255, 255, 255, 0.1) !important;
}

.tab-content :deep(.action-card.danger) {
  background: rgba(239, 83, 80, 0.1) !important;
  border-color: rgba(239, 83, 80, 0.3) !important;
}

.tab-content :deep(.action-card h4) {
  color: #e0e0e0 !important;
}

.tab-content :deep(.action-card p) {
  color: #888 !important;
}

.tab-content :deep(.section-description) {
  color: #888 !important;
}

.tab-content :deep(small) {
  color: #666 !important;
}

/* Outlined buttons */
.tab-content :deep(.p-button-outlined) {
  background: transparent !important;
  color: #4fc3f7 !important;
  border-color: #4fc3f7 !important;
}

.tab-content :deep(.p-button-outlined:hover) {
  background: rgba(79, 195, 247, 0.1) !important;
}

.tab-content :deep(.p-button-danger.p-button-outlined) {
  color: #ef5350 !important;
  border-color: #ef5350 !important;
}

.tab-content :deep(.p-button-danger.p-button-outlined:hover) {
  background: rgba(239, 83, 80, 0.1) !important;
}

/* Chips input (departments, permissions) */
.tab-content :deep(.p-chips-input) {
  background: rgba(255, 255, 255, 0.08) !important;
  border-color: rgba(255, 255, 255, 0.15) !important;
}

.tab-content :deep(.p-chips .p-inputtext) {
  background: transparent !important;
  border: none !important;
  color: #e0e0e0 !important;
}
</style>
