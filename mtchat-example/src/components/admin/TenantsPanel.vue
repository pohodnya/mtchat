<template>
  <div class="panel">
    <!-- Create Form -->
    <div class="form-section">
      <h3>Create Tenant</h3>
      <div class="form-row">
        <InputText
          v-model="newTenantName"
          placeholder="Tenant name (e.g., Acme Corp)"
          class="flex-1"
          @keyup.enter="handleCreate"
        />
        <Button
          label="Create"
          icon="pi pi-plus"
          @click="handleCreate"
          :disabled="!newTenantName.trim()"
        />
      </div>
    </div>

    <!-- List -->
    <div class="list-section">
      <h3>Tenants ({{ tenants.length }})</h3>
      <DataTable
        :value="sortedTenants"
        stripedRows
        :paginator="tenants.length > 10"
        :rows="10"
        emptyMessage="No tenants yet"
      >
        <Column field="name" header="Name" sortable />
        <Column field="id" header="ID">
          <template #body="{ data }">
            <code class="uuid">{{ data.id }}</code>
          </template>
        </Column>
        <Column field="createdAt" header="Created" sortable>
          <template #body="{ data }">
            {{ formatDate(data.createdAt) }}
          </template>
        </Column>
        <Column header="Actions" style="width: 100px">
          <template #body="{ data }">
            <Button
              icon="pi pi-trash"
              severity="danger"
              text
              rounded
              @click="handleDelete(data)"
              v-tooltip="'Delete'"
            />
          </template>
        </Column>
      </DataTable>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useToast } from 'primevue/usetoast'
import InputText from 'primevue/inputtext'
import Button from 'primevue/button'
import DataTable from 'primevue/datatable'
import Column from 'primevue/column'
import { useTenants } from '../../composables'
import type { Tenant } from '../../types'

const toast = useToast()
const { tenants, sortedTenants, createTenant, deleteTenant } = useTenants()

const newTenantName = ref('')

function handleCreate() {
  if (!newTenantName.value.trim()) return

  const tenant = createTenant(newTenantName.value)
  newTenantName.value = ''

  toast.add({
    severity: 'success',
    summary: 'Created',
    detail: `Tenant "${tenant.name}" created`,
    life: 3000,
  })
}

function handleDelete(tenant: Tenant) {
  deleteTenant(tenant.id)
  toast.add({
    severity: 'info',
    summary: 'Deleted',
    detail: `Tenant "${tenant.name}" deleted`,
    life: 3000,
  })
}

function formatDate(dateString: string): string {
  return new Date(dateString).toLocaleDateString()
}
</script>

<style scoped>
.panel {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.form-section,
.list-section {
  background: white;
  border-radius: 8px;
  padding: 20px;
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.05);
}

h3 {
  font-size: 14px;
  font-weight: 600;
  color: #666;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  margin-bottom: 16px;
}

.form-row {
  display: flex;
  gap: 12px;
}

.flex-1 {
  flex: 1;
}

.uuid {
  font-family: 'SF Mono', Monaco, monospace;
  font-size: 11px;
  background: #f0f0f0;
  padding: 2px 6px;
  border-radius: 4px;
}
</style>
