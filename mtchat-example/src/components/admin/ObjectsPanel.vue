<template>
  <div class="panel">
    <!-- Create Form -->
    <div class="form-section">
      <h3>Create Mock Object</h3>
      <div class="form-grid">
        <div class="form-field">
          <label>Type</label>
          <InputText
            v-model="form.type"
            placeholder="e.g. project, task, ticket"
            class="w-full"
          />
        </div>
        <div class="form-field">
          <label>Title</label>
          <InputText
            v-model="form.title"
            placeholder="Object title"
            class="w-full"
          />
        </div>
        <div class="form-field full-width">
          <label>Description</label>
          <Textarea
            v-model="form.description"
            placeholder="Description (optional)"
            rows="2"
            class="w-full"
          />
        </div>
      </div>
      <Button
        label="Create Object"
        icon="pi pi-plus"
        class="mt-3"
        @click="handleCreate"
        :disabled="!form.type || !form.title.trim()"
      />
    </div>

    <!-- Objects List -->
    <div class="list-section">
      <h3>Objects ({{ objects.length }})</h3>

      <DataTable
        v-if="objects.length > 0"
        :value="objects"
        stripedRows
        emptyMessage="No objects"
      >
        <Column field="type" header="Type" sortable>
          <template #body="{ data }">
            <span class="type-badge">{{ data.type }}</span>
          </template>
        </Column>
        <Column field="title" header="Title" sortable />
        <Column field="description" header="Description">
          <template #body="{ data }">
            <span class="description">{{ data.description || '-' }}</span>
          </template>
        </Column>
        <Column field="id" header="ID">
          <template #body="{ data }">
            <code class="uuid">{{ data.id.slice(0, 8) }}...</code>
          </template>
        </Column>
        <Column field="createdAt" header="Created">
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

      <div v-else class="empty-state">
        <i class="pi pi-inbox" />
        <p>No objects yet. Create one to use in inline mode.</p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { reactive } from 'vue'
import { useToast } from 'primevue/usetoast'
import InputText from 'primevue/inputtext'
import Textarea from 'primevue/textarea'
import Button from 'primevue/button'
import DataTable from 'primevue/datatable'
import Column from 'primevue/column'
import { useObjects } from '../../composables'
import type { MockObject } from '../../types'

const toast = useToast()
const { objects, createObject, deleteObject } = useObjects()

const form = reactive({
  type: '',
  title: '',
  description: '',
})

function handleCreate() {
  if (!form.type || !form.title.trim()) return

  const obj = createObject({
    type: form.type,
    title: form.title,
    description: form.description,
  })

  // Reset form
  form.title = ''
  form.description = ''

  toast.add({
    severity: 'success',
    summary: 'Created',
    detail: `Object "${obj.title}" created`,
    life: 3000,
  })
}

function handleDelete(obj: MockObject) {
  deleteObject(obj.id)
  toast.add({
    severity: 'info',
    summary: 'Deleted',
    detail: `Object "${obj.title}" deleted`,
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
  color: #374151;
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

.form-field.full-width {
  grid-column: 1 / -1;
}

.form-field label {
  font-size: 13px;
  font-weight: 500;
  color: #374151;
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
  background: #e2e8f0;
  color: #334155;
  padding: 2px 6px;
  border-radius: 4px;
}

.type-badge {
  display: inline-block;
  padding: 2px 8px;
  border-radius: 4px;
  font-size: 12px;
  font-weight: 500;
  background: rgba(59, 130, 246, 0.15);
  color: #3b82f6;
}

.description {
  color: #475569;
  font-size: 13px;
  max-width: 300px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.empty-state {
  text-align: center;
  padding: 40px;
  color: #64748b;
}

.empty-state i {
  font-size: 48px;
  margin-bottom: 16px;
}
</style>
