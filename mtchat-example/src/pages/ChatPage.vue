<template>
  <TMSLayout :show-banner="false">
    <!-- No User Selected -->
    <div v-if="!currentUser" class="no-user-overlay">
      <div class="no-user-content">
        <i class="pi pi-user" />
        <h2>No User Selected</h2>
        <p>Select a user from the dropdown in the header to enter the chat</p>
        <p v-if="users.length === 0" class="hint">
          First, create users in the
          <router-link to="/admin">Admin Panel</router-link>
        </p>
      </div>
    </div>

    <!-- Chat Component (Full Mode) -->
    <div v-else class="chat-full-wrapper">
      <MTChat
        :key="currentUser.id"
        :config="chatConfig"
        mode="full"
        :show-header="true"
        :show-sidebar="true"
        :theme="settings.theme"
        @connected="onConnected"
        @disconnected="onDisconnected"
        @error="onError"
        @message-sent="onMessageSent"
      />
    </div>
  </TMSLayout>
  <Toast />
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { useToast } from 'primevue/usetoast'
import Toast from 'primevue/toast'
import { MTChat, type MTChatConfig, type Message } from '@mtchat/vue'
import TMSLayout from '../components/TMSLayout.vue'
import { useUsers, useSettings } from '../composables'

const toast = useToast()
const { users, currentUser } = useUsers()
const { settings } = useSettings()

// Build chat config from current user
const chatConfig = computed<MTChatConfig>(() => {
  if (!currentUser.value) {
    return {
      baseUrl: settings.value.apiBaseUrl,
      userId: '',
      scopeConfig: {
        tenant_uid: '',
        scope_level1: [],
        scope_level2: [],
      },
    }
  }

  return {
    baseUrl: settings.value.apiBaseUrl,
    userId: currentUser.value.id,
    scopeConfig: {
      tenant_uid: currentUser.value.tenantId,
      scope_level1: currentUser.value.scopeLevel1,
      scope_level2: currentUser.value.scopeLevel2,
    },
  }
})

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
  toast.add({
    severity: 'error',
    summary: 'Error',
    detail: error.message,
    life: 5000,
  })
}

function onMessageSent(message: Message) {
  console.log('Message sent:', message)
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
  color: var(--tms-text-secondary, #888);
}

.no-user-content i {
  font-size: 64px;
  color: var(--tms-text-muted, #444);
  margin-bottom: 16px;
  display: block;
}

.no-user-content h2 {
  font-size: 24px;
  font-weight: 600;
  color: var(--tms-text, #e0e0e0);
  margin-bottom: 8px;
}

.no-user-content p {
  color: var(--tms-text-secondary, #888);
  margin-bottom: 8px;
}

.no-user-content .hint {
  font-size: 14px;
  color: var(--tms-text-muted, #666);
}

.no-user-content a {
  color: var(--tms-primary, #4fc3f7);
  text-decoration: none;
}

.no-user-content a:hover {
  text-decoration: underline;
}

.chat-full-wrapper {
  flex: 1;
  display: flex;
  padding: 24px;
  min-height: 0;
}

.chat-full-wrapper > :deep(*) {
  flex: 1;
  border-radius: 12px;
  overflow: hidden;
  box-shadow: 0 4px 24px rgba(0, 0, 0, 0.3);
}
</style>
