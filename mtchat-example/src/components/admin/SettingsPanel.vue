<template>
  <div class="panel">
    <!-- Settings Form -->
    <div class="form-section">
      <h3>API Settings</h3>
      <div class="form-grid">
        <div class="form-field">
          <label>Admin Token</label>
          <InputText
            v-model="localSettings.adminToken"
            placeholder="Admin token for Management API"
            class="w-full"
            type="password"
          />
          <small>Required for creating dialogs via Management API</small>
        </div>
        <div class="form-field">
          <label>API Base URL</label>
          <InputText
            v-model="localSettings.apiBaseUrl"
            placeholder="http://localhost:8081"
            class="w-full"
          />
        </div>
      </div>
      <Button
        label="Save Settings"
        icon="pi pi-save"
        class="mt-3"
        @click="saveSettings"
      />
    </div>

    <!-- Data Management -->
    <div class="form-section">
      <h3>Data Management</h3>
      <div class="data-actions">
        <div class="action-card">
          <h4>Export Data</h4>
          <p>Download all localStorage data as JSON file</p>
          <Button
            label="Export"
            icon="pi pi-download"
            outlined
            @click="exportData"
          />
        </div>
        <div class="action-card">
          <h4>Import Data</h4>
          <p>Restore data from a previously exported JSON file</p>
          <input
            ref="importInput"
            type="file"
            accept=".json"
            style="display: none"
            @change="handleImport"
          />
          <Button
            label="Import"
            icon="pi pi-upload"
            outlined
            @click="triggerImport"
          />
        </div>
        <div class="action-card danger">
          <h4>Reset All Data</h4>
          <p>Clear all localStorage data (tenants, users, objects, dialogs)</p>
          <Button
            label="Reset"
            icon="pi pi-trash"
            severity="danger"
            outlined
            @click="confirmReset"
          />
        </div>
      </div>
    </div>

    <!-- Sample Data -->
    <div class="form-section">
      <h3>Sample Data</h3>
      <p class="section-description">
        Generate sample data to quickly test the demo application
      </p>
      <Button
        label="Generate Sample Data"
        icon="pi pi-database"
        outlined
        @click="generateSampleData"
      />
    </div>

    <ConfirmDialog />
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, watch } from 'vue'
import { useToast } from 'primevue/usetoast'
import { useConfirm } from 'primevue/useconfirm'
import InputText from 'primevue/inputtext'
import Button from 'primevue/button'
import ConfirmDialog from 'primevue/confirmdialog'
import { useSettings, useTenants, useUsers, useObjects, useDialogRefs, generateUUID } from '../../composables'
import { STORAGE_KEYS, DEFAULT_SETTINGS } from '../../types'

const toast = useToast()
const confirm = useConfirm()
const { settings, updateSettings, resetSettings } = useSettings()
const { tenants, createTenant, clearTenants } = useTenants()
const { users, createUser, clearUsers } = useUsers()
const { objects, createObject, clearObjects } = useObjects()
const { dialogRefs, clearDialogRefs } = useDialogRefs()

const importInput = ref<HTMLInputElement>()

// Local copy of settings for editing
const localSettings = reactive({
  adminToken: settings.value.adminToken,
  apiBaseUrl: settings.value.apiBaseUrl,
})

// Sync when settings change externally
watch(settings, (newVal) => {
  localSettings.adminToken = newVal.adminToken
  localSettings.apiBaseUrl = newVal.apiBaseUrl
})

function saveSettings() {
  updateSettings({
    adminToken: localSettings.adminToken,
    apiBaseUrl: localSettings.apiBaseUrl,
  })
  toast.add({
    severity: 'success',
    summary: 'Saved',
    detail: 'Settings saved',
    life: 3000,
  })
}

function exportData() {
  const data = {
    version: 1,
    exportedAt: new Date().toISOString(),
    tenants: tenants.value,
    users: users.value,
    objects: objects.value,
    dialogRefs: dialogRefs.value,
    settings: settings.value,
  }

  const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `mtchat-demo-export-${new Date().toISOString().slice(0, 10)}.json`
  a.click()
  URL.revokeObjectURL(url)

  toast.add({
    severity: 'success',
    summary: 'Exported',
    detail: 'Data exported to file',
    life: 3000,
  })
}

function triggerImport() {
  importInput.value?.click()
}

function handleImport(event: Event) {
  const file = (event.target as HTMLInputElement).files?.[0]
  if (!file) return

  const reader = new FileReader()
  reader.onload = (e) => {
    try {
      const data = JSON.parse(e.target?.result as string)

      // Validate structure
      if (!data.version || !Array.isArray(data.tenants)) {
        throw new Error('Invalid file format')
      }

      // Import data
      localStorage.setItem(STORAGE_KEYS.TENANTS, JSON.stringify(data.tenants || []))
      localStorage.setItem(STORAGE_KEYS.USERS, JSON.stringify(data.users || []))
      localStorage.setItem(STORAGE_KEYS.OBJECTS, JSON.stringify(data.objects || []))
      localStorage.setItem(STORAGE_KEYS.DIALOG_REFS, JSON.stringify(data.dialogRefs || []))
      if (data.settings) {
        localStorage.setItem(STORAGE_KEYS.SETTINGS, JSON.stringify(data.settings))
      }

      // Reload page to pick up changes
      window.location.reload()
    } catch (err) {
      toast.add({
        severity: 'error',
        summary: 'Error',
        detail: 'Invalid file format',
        life: 5000,
      })
    }
  }
  reader.readAsText(file)
}

function confirmReset() {
  confirm.require({
    message: 'Are you sure you want to delete all data? This cannot be undone.',
    header: 'Reset All Data',
    icon: 'pi pi-exclamation-triangle',
    rejectLabel: 'Cancel',
    acceptLabel: 'Reset',
    acceptClass: 'p-button-danger',
    accept: () => {
      clearTenants()
      clearUsers()
      clearObjects()
      clearDialogRefs()
      resetSettings()
      localSettings.adminToken = DEFAULT_SETTINGS.adminToken
      localSettings.apiBaseUrl = DEFAULT_SETTINGS.apiBaseUrl

      toast.add({
        severity: 'info',
        summary: 'Reset',
        detail: 'All data cleared',
        life: 3000,
      })
    },
  })
}

function generateSampleData() {
  // Create tenants
  const tenant1 = createTenant('Acme Corp')
  const tenant2 = createTenant('Logistics Pro')

  // Create users
  createUser({
    name: 'Alice Manager',
    tenantId: tenant1.id,
    scopeLevel1: ['sales', 'logistics'],
    scopeLevel2: ['manager', 'viewer'],
  })
  createUser({
    name: 'Bob Dispatcher',
    tenantId: tenant1.id,
    scopeLevel1: ['logistics'],
    scopeLevel2: ['dispatcher'],
  })
  createUser({
    name: 'Carol Admin',
    tenantId: tenant2.id,
    scopeLevel1: ['operations'],
    scopeLevel2: ['admin', 'manager'],
  })
  createUser({
    name: 'Dave Driver',
    tenantId: tenant2.id,
    scopeLevel1: ['drivers'],
    scopeLevel2: ['viewer'],
  })

  // Create objects
  createObject({
    type: 'tender',
    title: 'Tender #T-2024-001',
    description: 'Freight from Moscow to St. Petersburg, 20 tons',
  })
  createObject({
    type: 'tender',
    title: 'Tender #T-2024-002',
    description: 'Express delivery, Kazan to Samara',
  })
  createObject({
    type: 'order',
    title: 'Order #O-5678',
    description: 'Confirmed shipment, container freight',
  })
  createObject({
    type: 'route',
    title: 'Route R-001',
    description: 'Weekly route: Moscow - Nizhny Novgorod - Kazan',
  })

  toast.add({
    severity: 'success',
    summary: 'Generated',
    detail: 'Sample data created (2 tenants, 4 users, 4 objects)',
    life: 5000,
  })
}
</script>

<style scoped>
.panel {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.form-section {
  background: white;
  border-radius: 8px;
  padding: 20px;
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.05);
}

h3 {
  font-size: 14px;
  font-weight: 600;
  color: #374151;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  margin-bottom: 16px;
}

.section-description {
  color: #475569;
  margin-bottom: 16px;
}

.form-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 16px;
}

.form-field {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.form-field label {
  font-size: 13px;
  font-weight: 500;
  color: #374151;
}

.form-field small {
  font-size: 12px;
  color: #64748b;
}

.w-full {
  width: 100%;
}

.mt-3 {
  margin-top: 16px;
}

.data-actions {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 16px;
}

.action-card {
  padding: 16px;
  border: 1px solid #e0e0e0;
  border-radius: 8px;
}

.action-card.danger {
  border-color: #ffcdd2;
  background: #fff5f5;
}

.action-card h4 {
  font-size: 14px;
  font-weight: 600;
  color: #1e293b;
  margin-bottom: 8px;
}

.action-card p {
  font-size: 13px;
  color: #475569;
  margin-bottom: 12px;
}
</style>
