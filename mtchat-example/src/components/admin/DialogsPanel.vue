<template>
  <div class="panel">
    <!-- Create Form -->
    <div class="form-section">
      <h3>Create Dialog (via Management API)</h3>

      <Message v-if="!settings.adminToken" severity="warn" :closable="false">
        Set Admin Token in Settings tab to create dialogs
      </Message>

      <div v-else class="form-grid">
        <!-- Object Selection -->
        <div class="form-field">
          <label>Object</label>
          <Select
            v-model="form.objectId"
            :options="objectOptions"
            optionLabel="label"
            optionValue="value"
            placeholder="Select object"
            class="w-full"
          />
        </div>

        <!-- Title -->
        <div class="form-field">
          <label>Title (optional)</label>
          <InputText
            v-model="form.title"
            placeholder="Dialog title"
            class="w-full"
          />
        </div>

        <!-- Participants -->
        <div class="form-field full-width">
          <label>Direct Participants</label>
          <MultiSelect
            v-model="form.participantIds"
            :options="userOptions"
            optionLabel="label"
            optionValue="value"
            placeholder="Select users"
            class="w-full"
            display="chip"
          />
        </div>

        <!-- Access Scopes -->
        <div class="form-field full-width">
          <label>Access Scopes (potential participants)</label>
          <div class="scopes-list">
            <div
              v-for="(scope, index) in form.accessScopes"
              :key="index"
              class="scope-row"
            >
              <Select
                v-model="scope.tenantId"
                :options="tenantOptions"
                optionLabel="label"
                optionValue="value"
                placeholder="Tenant"
                class="scope-select"
              />
              <Chips
                v-model="scope.scopeLevel1"
                placeholder="Departments (Enter to add)"
                separator=","
                :addOnBlur="true"
                class="scope-chips"
              />
              <Chips
                v-model="scope.scopeLevel2"
                placeholder="Permissions (Enter to add)"
                separator=","
                :addOnBlur="true"
                class="scope-chips"
              />
              <Button
                icon="pi pi-times"
                severity="danger"
                text
                rounded
                @click="removeScope(index)"
              />
            </div>
            <Button
              label="Add Scope Rule"
              icon="pi pi-plus"
              text
              size="small"
              @click="addScope"
            />
          </div>
        </div>
      </div>

      <Button
        label="Create Dialog"
        icon="pi pi-comments"
        class="mt-3"
        @click="handleCreate"
        :disabled="!canCreate"
        :loading="creating"
      />
    </div>

    <!-- List -->
    <div class="list-section">
      <h3>Created Dialogs ({{ dialogRefs.length }})</h3>

      <DataTable
        :value="sortedDialogRefs"
        stripedRows
        :paginator="dialogRefs.length > 10"
        :rows="10"
        emptyMessage="No dialogs created yet"
      >
        <Column header="Object">
          <template #body="{ data }">
            <Tag :value="data.objectType" severity="info" />
            <span class="ml-2">{{ getObjectTitle(data.objectId) }}</span>
          </template>
        </Column>
        <Column field="title" header="Title">
          <template #body="{ data }">
            {{ data.title || '-' }}
          </template>
        </Column>
        <Column header="Participants">
          <template #body="{ data }">
            <span>{{ data.participants.length }} users</span>
          </template>
        </Column>
        <Column header="Scopes">
          <template #body="{ data }">
            <span>{{ data.accessScopes.length }} rules</span>
          </template>
        </Column>
        <Column field="id" header="ID">
          <template #body="{ data }">
            <code class="uuid">{{ data.id.slice(0, 8) }}...</code>
          </template>
        </Column>
        <Column header="Actions" style="width: 120px">
          <template #body="{ data }">
            <Button
              icon="pi pi-trash"
              severity="danger"
              text
              rounded
              @click="handleDelete(data)"
              v-tooltip="'Delete'"
              :loading="deleting === data.id"
            />
          </template>
        </Column>
      </DataTable>
    </div>
  </div>
</template>

<script setup lang="ts">
import { reactive, computed, ref } from 'vue'
import { useToast } from 'primevue/usetoast'
import InputText from 'primevue/inputtext'
import Select from 'primevue/select'
import MultiSelect from 'primevue/multiselect'
import Chips from 'primevue/chips'
import Button from 'primevue/button'
import DataTable from 'primevue/datatable'
import Column from 'primevue/column'
import Tag from 'primevue/tag'
import Message from 'primevue/message'
import { useTenants, useUsers, useObjects, useDialogRefs, useSettings } from '../../composables'
import { ManagementApi, type ParticipantInput } from '../../services/managementApi'
import type { DialogRef } from '../../types'

const toast = useToast()
const { sortedTenants, getTenant } = useTenants()
const { sortedUsers, getUser } = useUsers()
const { objects, getObject } = useObjects()
const { dialogRefs, sortedDialogRefs, addDialogRef, removeDialogRef } = useDialogRefs()
const { settings } = useSettings()

const creating = ref(false)
const deleting = ref<string | null>(null)

// Form state
interface ScopeForm {
  tenantId: string
  scopeLevel1: string[]
  scopeLevel2: string[]
}

const form = reactive({
  objectId: '',
  title: '',
  participantIds: [] as string[],
  accessScopes: [] as ScopeForm[],
})

// Options for selects
const objectOptions = computed(() =>
  objects.value.map((o) => ({
    label: `[${o.type}] ${o.title}`,
    value: o.id,
    type: o.type,
  }))
)

const userOptions = computed(() =>
  sortedUsers.value.map((u) => ({
    label: u.name,
    value: u.id,
  }))
)

const tenantOptions = computed(() =>
  sortedTenants.value.map((t) => ({
    label: t.name,
    value: t.id,
  }))
)

const canCreate = computed(() => {
  return settings.value.adminToken && form.objectId
})

function addScope() {
  form.accessScopes.push({
    tenantId: '',
    scopeLevel1: [],
    scopeLevel2: [],
  })
}

function removeScope(index: number) {
  form.accessScopes.splice(index, 1)
}

function getObjectTitle(objectId: string): string {
  return getObject(objectId)?.title || objectId.slice(0, 8)
}

async function handleCreate() {
  if (!canCreate.value) return

  const selectedObject = getObject(form.objectId)
  if (!selectedObject) {
    toast.add({
      severity: 'error',
      summary: 'Error',
      detail: 'Object not found',
      life: 3000,
    })
    return
  }

  creating.value = true
  try {
    const api = new ManagementApi(settings.value.apiBaseUrl, settings.value.adminToken)

    // Build participant profiles from selected user IDs
    const participants: ParticipantInput[] = form.participantIds
      .map((userId) => {
        const user = getUser(userId)
        if (!user) return null
        const tenant = getTenant(user.tenantId)
        return {
          user_id: user.id,
          display_name: user.name,
          company: tenant?.name,
          email: user.email,
          phone: user.phone,
        } as ParticipantInput
      })
      .filter((p): p is ParticipantInput => p !== null)

    // Build request
    const request = {
      object_id: selectedObject.id,
      object_type: selectedObject.type,
      title: form.title || undefined,
      participants: participants.length > 0 ? participants : undefined,
      access_scopes: form.accessScopes
        .filter((s) => s.tenantId)
        .map((s) => ({
          tenant_uid: s.tenantId,
          scope_level1: s.scopeLevel1,
          scope_level2: s.scopeLevel2,
        })),
    }

    const dialog = await api.createDialog(request)

    // Save reference
    addDialogRef({
      id: dialog.id,
      objectId: selectedObject.id,
      objectType: selectedObject.type,
      title: form.title,
      participants: form.participantIds,
      accessScopes: form.accessScopes
        .filter((s) => s.tenantId)
        .map((s) => ({
          tenantUid: s.tenantId,
          scopeLevel1: s.scopeLevel1,
          scopeLevel2: s.scopeLevel2,
        })),
    })

    // Reset form
    form.objectId = ''
    form.title = ''
    form.participantIds = []
    form.accessScopes = []

    toast.add({
      severity: 'success',
      summary: 'Created',
      detail: `Dialog created with ID ${dialog.id.slice(0, 8)}...`,
      life: 3000,
    })
  } catch (e) {
    toast.add({
      severity: 'error',
      summary: 'Error',
      detail: e instanceof Error ? e.message : 'Failed to create dialog',
      life: 5000,
    })
  } finally {
    creating.value = false
  }
}

async function handleDelete(dialogRef: DialogRef) {
  deleting.value = dialogRef.id
  try {
    const api = new ManagementApi(settings.value.apiBaseUrl, settings.value.adminToken)
    await api.deleteDialog(dialogRef.id)
    removeDialogRef(dialogRef.id)

    toast.add({
      severity: 'info',
      summary: 'Deleted',
      detail: 'Dialog deleted',
      life: 3000,
    })
  } catch (e) {
    toast.add({
      severity: 'error',
      summary: 'Error',
      detail: e instanceof Error ? e.message : 'Failed to delete dialog',
      life: 5000,
    })
  } finally {
    deleting.value = null
  }
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

.ml-2 {
  margin-left: 8px;
}

.uuid {
  font-family: 'SF Mono', Monaco, monospace;
  font-size: 11px;
  background: #e2e8f0;
  color: #334155;
  padding: 2px 6px;
  border-radius: 4px;
}

.scopes-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.scope-row {
  display: flex;
  gap: 8px;
  align-items: flex-start;
}

.scope-select {
  width: 180px;
}

.scope-chips {
  flex: 1;
}
</style>
