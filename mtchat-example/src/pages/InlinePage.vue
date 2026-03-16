<template>
  <DemoLayout :show-banner="false">
    <!-- Main Content: Data Table -->
    <template v-if="currentUser">
      <DemoDataTable
        :objects="sortedObjects"
        :selected-id="selectedObjectId"
        @row-click="handleRowClick"
      />
    </template>

    <!-- No User Selected -->
    <div v-else class="no-user-overlay">
      <div class="no-user-content">
        <i class="pi pi-user" />
        <h2>No User Selected</h2>
        <p>Select a user from the dropdown in the header to view inline chats</p>
        <p v-if="users.length === 0" class="hint">
          First, create users in the
          <router-link to="/admin">Admin Panel</router-link>
        </p>
      </div>
    </div>

    <!-- Right Panel: Chat -->
    <template #rightPanel v-if="currentUser && selectedObject">
      <DemoChatPanel
        :title="selectedObject.title"
        :subtitle="selectedObject.type"
        :tabs="['Информация', 'Маршрут', 'Чат']"
        @close="selectedObjectId = null"
      >
        <!-- Waiting for JWT -->
        <div v-if="!isJwtReady" class="jwt-loading">
          <i class="pi pi-spin pi-spinner" />
          <span>Loading...</span>
        </div>
        <MTChatPrime
          v-else
          :key="chatKey"
          :config="chatConfig"
          mode="inline"
          :object-type="selectedObject.type"
          :object-id="selectedObject.id"
          :show-header="true"
          :show-sidebar="false"
          :theme="settings.theme"
          @connected="onConnected"
          @disconnected="onDisconnected"
          @error="onError"
        />
      </DemoChatPanel>
    </template>
  </DemoLayout>
  <Toast />
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useToast } from 'primevue/usetoast'
import Toast from 'primevue/toast'
import { MTChatPrime, MTChatConfig } from '@mtchat/vue-primevue'
import DemoLayout from '../components/DemoLayout.vue'
import DemoDataTable from '../components/DemoDataTable.vue'
import DemoChatPanel from '../components/DemoChatPanel.vue'
import { useUsers, useObjects, useSettings, useTenants, useJwt } from '../composables'
import type { MockObject } from '../types'

const route = useRoute()
const router = useRouter()
const toast = useToast()

const { users, currentUser } = useUsers()
const { sortedObjects, getObject } = useObjects()
const { settings } = useSettings()
const { getTenant } = useTenants()
const { token: jwtToken } = useJwt()

// JWT ready check: either JWT disabled, or token generated
const isJwtReady = computed(() => {
  if (!settings.value.jwtEnabled) return true
  return !!jwtToken.value
})

// Selected object
const selectedObjectId = ref<string | null>(null)

// Initialize from route params
if (route.params.objectId) {
  selectedObjectId.value = route.params.objectId as string
}

const selectedObject = computed(() =>
  selectedObjectId.value ? getObject(selectedObjectId.value) : null
)

// Key for remounting MTChat when object or user changes
const chatKey = computed(() => {
  const objKey = selectedObject.value ? `${selectedObject.value.type}-${selectedObject.value.id}` : 'none'
  const userKey = currentUser.value?.id || 'no-user'
  return `${userKey}-${objKey}`
})

// Build chat config from current user
const chatConfig = computed<MTChatConfig>(() => {
  if (!currentUser.value) {
    return {
      baseUrl: settings.value.apiBaseUrl,
      userId: '',
      scopeConfig: {
        scopeLevel0: [],
        scopeLevel1: [],
        scopeLevel2: [],
      },
      userProfile: {
        displayName: '',
        company: '',
      },
    }
  }

  const tenant = getTenant(currentUser.value.tenantId)

  return {
    baseUrl: settings.value.apiBaseUrl,
    userId: currentUser.value.id,
    token: jwtToken.value,
    scopeConfig: {
      scopeLevel0: [currentUser.value.tenantId],
      scopeLevel1: currentUser.value.scopeLevel1,
      scopeLevel2: currentUser.value.scopeLevel2,
    },
    userProfile: {
      displayName: currentUser.value.name,
      company: tenant?.name || 'Unknown Company',
      email: currentUser.value.email,
      phone: currentUser.value.phone,
    },
    locale: settings.value.locale,
  }
})

function handleRowClick(obj: MockObject) {
  selectedObjectId.value = obj.id
  router.replace({ path: `/inline/${obj.type}/${obj.id}` })
}

// Watch for route changes
watch(
  () => route.params.objectId,
  (newId) => {
    if (newId) {
      selectedObjectId.value = newId as string
    }
  }
)

// Event handlers
function onConnected() {
  toast.add({
    severity: 'success',
    summary: 'Connected',
    detail: 'WebSocket connected',
    life: 2000,
  })
}

function onDisconnected() {
  toast.add({
    severity: 'warn',
    summary: 'Disconnected',
    detail: 'WebSocket disconnected',
    life: 3000,
  })
}

function onError(error: Error) {
  // Don't show "no dialog" as error in inline mode
  if (error.message.includes('not found') || error.message.includes('404')) {
    return
  }
  toast.add({
    severity: 'error',
    summary: 'Error',
    detail: error.message,
    life: 5000,
  })
}
</script>

<style scoped>
.no-user-overlay {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
}

.no-user-content {
  text-align: center;
  color: var(--demo-text-secondary, #888);
}

.no-user-content i {
  font-size: 64px;
  color: var(--demo-text-muted, #444);
  margin-bottom: 16px;
  display: block;
}

.no-user-content h2 {
  font-size: 24px;
  font-weight: 600;
  color: var(--demo-text, #e0e0e0);
  margin-bottom: 8px;
}

.no-user-content p {
  color: var(--demo-text-secondary, #888);
  margin-bottom: 8px;
}

.no-user-content .hint {
  font-size: 14px;
  color: var(--demo-text-muted, #666);
}

.no-user-content a {
  color: var(--demo-primary, #4fc3f7);
  text-decoration: none;
}

.no-user-content a:hover {
  text-decoration: underline;
}

.jwt-loading {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  gap: 12px;
  color: var(--demo-text-secondary, #888);
}

.jwt-loading i {
  font-size: 24px;
}
</style>
