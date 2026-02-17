<template>
  <DemoLayout :show-banner="false">
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
      <MTChatPrime
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
      >
        <!-- Custom actions in header menu (before "Leave chat") -->
        <template #header-menu-actions="{ dialog, closeMenu, menuItemClass }">
          <button
            :class="menuItemClass"
            @click="onHeaderMenuAction(dialog); closeMenu()"
          >
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="12" cy="12" r="10"/>
              <path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3"/>
              <line x1="12" y1="17" x2="12.01" y2="17"/>
            </svg>
            Действие 1
          </button>
          <button
            :class="menuItemClass"
            @click="console.log('Action 2', dialog.id); closeMenu()"
          >
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M12 2v4M12 18v4M4.93 4.93l2.83 2.83M16.24 16.24l2.83 2.83M2 12h4M18 12h4M4.93 19.07l2.83-2.83M16.24 7.76l2.83-2.83"/>
            </svg>
            Действие 2
          </button>
        </template>

        <!-- Custom action button in sidebar -->
        <template #sidebar-action>
          <button
            class="create-chat-btn"
            type="button"
            title="Create chat"
            @click="showCreateDialog = true"
          >
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M12 5v14M5 12h14"/>
            </svg>
          </button>
        </template>
      </MTChatPrime>
    </div>

    <!-- Create Chat Dialog -->
    <Dialog
      v-model:visible="showCreateDialog"
      header="Create Chat"
      :modal="true"
      :style="{ width: '400px' }"
    >
      <p class="create-dialog-placeholder">
        Chat creation form will be here
      </p>
    </Dialog>

    <!-- Stub Dialog -->
    <Dialog
      v-model:visible="showStubDialog"
      header="Заглушка"
      :modal="true"
      :closable="true"
      :style="{ width: '300px' }"
    >
      <p class="stub-dialog-content">
        Заглушка
      </p>
    </Dialog>
  </DemoLayout>
  <Toast />
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { useToast } from 'primevue/usetoast'
import Toast from 'primevue/toast'
import Dialog from 'primevue/dialog'
import { MTChatPrime, MTChatConfig, Message } from '@mtchat/vue-primevue'
import DemoLayout from '../components/DemoLayout.vue'
import { useUsers, useSettings, useTenants } from '../composables'

const toast = useToast()
const { users, currentUser } = useUsers()
const { settings } = useSettings()
const { getTenant } = useTenants()

// Create chat dialog
const showCreateDialog = ref(false)

// Stub dialog
const showStubDialog = ref(false)

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
    scopeConfig: {
      tenant_uid: currentUser.value.tenantId,
      scope_level1: currentUser.value.scopeLevel1,
      scope_level2: currentUser.value.scopeLevel2,
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

function onHeaderMenuAction(dialog: any) {
  console.log('Header menu action for dialog:', dialog.id)
  showStubDialog.value = true
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

/* Create chat button (in sidebar slot) */
.create-chat-btn {
  width: 36px;
  height: 36px;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 1px solid var(--p-surface-border, #3f3f46);
  border-radius: 8px;
  background: transparent;
  color: var(--p-text-muted-color, #a1a1aa);
  cursor: pointer;
  transition: all 0.15s;
}

.create-chat-btn:hover {
  border-color: var(--p-primary-color, #3b82f6);
  color: var(--p-primary-color, #3b82f6);
  background: var(--p-surface-hover, rgba(255, 255, 255, 0.05));
}

.create-dialog-placeholder {
  color: var(--p-text-muted-color, #a1a1aa);
  text-align: center;
  padding: 24px;
  margin: 0;
}

.stub-dialog-content {
  text-align: center;
  padding: 24px;
  margin: 0;
  color: var(--p-text-color, #e0e0e0);
}
</style>
