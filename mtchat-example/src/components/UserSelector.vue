<template>
  <div class="user-selector" :class="{ compact }">
    <label v-if="compact" class="selector-label">Current User</label>
    <Select
      v-model="selectedUserId"
      :options="userOptions"
      optionLabel="label"
      optionValue="value"
      :placeholder="compact ? 'Select user' : 'Select user to login as'"
      class="user-select"
      @change="handleChange"
    >
      <template #value="{ value }">
        <div v-if="value && selectedUser" class="user-option">
          <span class="user-avatar">{{ selectedUser.name.charAt(0) }}</span>
          <span class="user-name">{{ selectedUser.name }}</span>
        </div>
        <span v-else class="placeholder">Select user</span>
      </template>
      <template #option="{ option }">
        <div class="user-option">
          <span class="user-avatar">{{ option.label.charAt(0) }}</span>
          <div class="user-info">
            <span class="user-name">{{ option.label }}</span>
            <small class="user-tenant">{{ option.tenant }}</small>
          </div>
        </div>
      </template>
    </Select>
    <Button
      v-if="currentUser"
      icon="pi pi-sign-out"
      text
      rounded
      @click="handleLogout"
      v-tooltip="'Logout'"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import Select from 'primevue/select'
import Button from 'primevue/button'
import { useUsers, useTenants } from '../composables'

withDefaults(defineProps<{
  compact?: boolean
}>(), {
  compact: false,
})

const { users, currentUser, setCurrentUser, getUser } = useUsers()
const { getTenant } = useTenants()

const selectedUserId = ref<string | null>(currentUser.value?.id || null)

const userOptions = computed(() =>
  users.value.map((u) => ({
    label: u.name,
    value: u.id,
    tenant: getTenant(u.tenantId)?.name || 'Unknown',
  }))
)

const selectedUser = computed(() =>
  selectedUserId.value ? getUser(selectedUserId.value) : null
)

// Sync with currentUser changes
watch(currentUser, (user) => {
  selectedUserId.value = user?.id || null
})

function handleChange() {
  if (selectedUserId.value) {
    const user = getUser(selectedUserId.value)
    if (user) {
      setCurrentUser(user)
    }
  }
}

function handleLogout() {
  selectedUserId.value = null
  setCurrentUser(null)
}
</script>

<style scoped>
.user-selector {
  display: flex;
  align-items: center;
  gap: 8px;
}

.user-select {
  min-width: 220px;
}

.user-option {
  display: flex;
  align-items: center;
  gap: 10px;
}

.user-avatar {
  width: 28px;
  height: 28px;
  border-radius: 50%;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 600;
  font-size: 12px;
  flex-shrink: 0;
}

.user-info {
  display: flex;
  flex-direction: column;
}

.user-name {
  font-weight: 500;
}

.user-tenant {
  font-size: 11px;
  color: var(--tms-text-muted, #999);
}

.placeholder {
  color: var(--tms-text-muted, #999);
}

/* Compact mode */
.user-selector.compact {
  flex-direction: column;
  align-items: stretch;
  gap: 6px;
}

.selector-label {
  font-size: 10px;
  font-weight: 600;
  color: var(--tms-text-secondary, #888);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.compact .user-select {
  min-width: unset;
}

.compact .user-avatar {
  width: 24px;
  height: 24px;
  font-size: 11px;
}
</style>
