<template>
  <div class="panel">
    <!-- Create Form -->
    <div class="form-section">
      <h3>Create User</h3>
      <div class="form-grid">
        <div class="form-field">
          <label>Name</label>
          <InputText
            v-model="form.name"
            placeholder="User name"
            class="w-full"
          />
        </div>
        <div class="form-field">
          <label>Tenant</label>
          <Select
            v-model="form.tenantId"
            :options="sortedTenants"
            optionLabel="name"
            optionValue="id"
            placeholder="Select tenant"
            class="w-full"
          />
        </div>
        <div class="form-field">
          <label>Departments (scope_level1)</label>
          <Chips
            v-model="form.scopeLevel1"
            placeholder="Press Enter to add"
            class="w-full"
          />
        </div>
        <div class="form-field">
          <label>Permissions (scope_level2)</label>
          <Chips
            v-model="form.scopeLevel2"
            placeholder="Press Enter to add"
            class="w-full"
          />
        </div>
      </div>
      <Button
        label="Create User"
        icon="pi pi-user-plus"
        class="mt-3"
        @click="handleCreate"
        :disabled="!form.name.trim() || !form.tenantId"
      />
    </div>

    <!-- List -->
    <div class="list-section">
      <h3>Users ({{ users.length }})</h3>
      <DataTable
        :value="sortedUsers"
        stripedRows
        :paginator="users.length > 10"
        :rows="10"
        emptyMessage="No users yet"
      >
        <Column field="name" header="Name" sortable />
        <Column header="Tenant">
          <template #body="{ data }">
            {{ getTenantName(data.tenantId) }}
          </template>
        </Column>
        <Column header="Departments">
          <template #body="{ data }">
            <div class="scope-chips">
              <Tag
                v-for="s in data.scopeLevel1"
                :key="s"
                :value="s"
                severity="info"
              />
              <span v-if="!data.scopeLevel1.length" class="empty-scope">-</span>
            </div>
          </template>
        </Column>
        <Column header="Permissions">
          <template #body="{ data }">
            <div class="scope-chips">
              <Tag
                v-for="s in data.scopeLevel2"
                :key="s"
                :value="s"
                severity="success"
              />
              <span v-if="!data.scopeLevel2.length" class="empty-scope">-</span>
            </div>
          </template>
        </Column>
        <Column field="id" header="ID">
          <template #body="{ data }">
            <code class="uuid">{{ data.id.slice(0, 8) }}...</code>
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
import { reactive } from 'vue'
import { useToast } from 'primevue/usetoast'
import InputText from 'primevue/inputtext'
import Select from 'primevue/select'
import Chips from 'primevue/chips'
import Button from 'primevue/button'
import DataTable from 'primevue/datatable'
import Column from 'primevue/column'
import Tag from 'primevue/tag'
import { useTenants, useUsers } from '../../composables'
import type { User } from '../../types'

const toast = useToast()
const { sortedTenants, getTenant } = useTenants()
const { users, sortedUsers, createUser, deleteUser } = useUsers()

const form = reactive({
  name: '',
  tenantId: '',
  scopeLevel1: [] as string[],
  scopeLevel2: [] as string[],
})

function handleCreate() {
  if (!form.name.trim() || !form.tenantId) return

  const user = createUser({
    name: form.name,
    tenantId: form.tenantId,
    scopeLevel1: form.scopeLevel1,
    scopeLevel2: form.scopeLevel2,
  })

  // Reset form
  form.name = ''
  form.scopeLevel1 = []
  form.scopeLevel2 = []

  toast.add({
    severity: 'success',
    summary: 'Created',
    detail: `User "${user.name}" created`,
    life: 3000,
  })
}

function handleDelete(user: User) {
  deleteUser(user.id)
  toast.add({
    severity: 'info',
    summary: 'Deleted',
    detail: `User "${user.name}" deleted`,
    life: 3000,
  })
}

function getTenantName(tenantId: string): string {
  return getTenant(tenantId)?.name || 'Unknown'
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
  color: #666;
}

.w-full {
  width: 100%;
}

.mt-3 {
  margin-top: 16px;
}

.uuid {
  font-family: 'SF Mono', Monaco, monospace;
  font-size: 11px;
  background: #f0f0f0;
  padding: 2px 6px;
  border-radius: 4px;
}

.scope-chips {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.empty-scope {
  color: #999;
}
</style>
