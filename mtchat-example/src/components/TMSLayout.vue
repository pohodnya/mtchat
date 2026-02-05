<template>
  <div class="tms-layout" :class="[`tms-layout--${currentTheme}`, { 'sidebar-collapsed': sidebarCollapsed, 'no-banner': !showBanner }]">
    <!-- Top Banner -->
    <div class="top-banner" v-if="showBanner">
      <span class="banner-content">Message Content</span>
      <div class="banner-actions">
        <button class="banner-btn">Button</button>
        <button class="banner-btn primary">Button</button>
      </div>
      <button class="banner-close" @click="$emit('close-banner')">
        <i class="pi pi-times" />
      </button>
    </div>

    <!-- Sidebar -->
    <aside class="sidebar">
      <div class="sidebar-header">
        <router-link to="/admin" class="logo">
          <span class="logo-icon">🚛</span>
          <span class="logo-text" v-if="!sidebarCollapsed">TRUCKER TMS</span>
        </router-link>
        <button class="sidebar-toggle" @click="sidebarCollapsed = !sidebarCollapsed">
          <i class="pi pi-bars" />
        </button>
      </div>

      <!-- Demo Navigation -->
      <div class="demo-nav" v-if="!sidebarCollapsed">
        <div class="demo-nav-label">MTChat Demo</div>
        <router-link to="/chat" class="demo-nav-link" :class="{ active: $route.path === '/chat' }">
          <i class="pi pi-comments" />
          Full Mode
        </router-link>
        <router-link to="/inline" class="demo-nav-link" :class="{ active: $route.path.startsWith('/inline') }">
          <i class="pi pi-window-maximize" />
          Inline Mode
        </router-link>
        <router-link to="/admin" class="demo-nav-link" :class="{ active: $route.path === '/admin' }">
          <i class="pi pi-cog" />
          Admin Panel
        </router-link>
        <button class="demo-nav-link theme-toggle" @click="toggleTheme">
          <i :class="currentTheme === 'dark' ? 'pi pi-sun' : 'pi pi-moon'" />
          {{ currentTheme === 'dark' ? 'Light Theme' : 'Dark Theme' }}
        </button>
      </div>
      <div class="demo-nav-collapsed" v-else>
        <router-link to="/chat" class="demo-nav-icon" :class="{ active: $route.path === '/chat' }" title="Full Mode">
          <i class="pi pi-comments" />
        </router-link>
        <router-link to="/inline" class="demo-nav-icon" :class="{ active: $route.path.startsWith('/inline') }" title="Inline Mode">
          <i class="pi pi-window-maximize" />
        </router-link>
        <router-link to="/admin" class="demo-nav-icon" :class="{ active: $route.path === '/admin' }" title="Admin Panel">
          <i class="pi pi-cog" />
        </router-link>
        <button class="demo-nav-icon theme-toggle" @click="toggleTheme" :title="currentTheme === 'dark' ? 'Switch to Light' : 'Switch to Dark'">
          <i :class="currentTheme === 'dark' ? 'pi pi-sun' : 'pi pi-moon'" />
        </button>
      </div>

      <div class="sidebar-divider" />

      <nav class="sidebar-nav">
        <div class="nav-group">
          <a href="#" class="nav-item">
            <i class="pi pi-bell" />
            <span v-if="!sidebarCollapsed">Уведомления</span>
            <span class="badge" v-if="!sidebarCollapsed">3</span>
          </a>
          <a href="#" class="nav-item">
            <i class="pi pi-question-circle" />
            <span v-if="!sidebarCollapsed">Техподдержка</span>
          </a>
          <a href="#" class="nav-item">
            <i class="pi pi-chart-line" />
            <span v-if="!sidebarCollapsed">Аналитика</span>
          </a>
        </div>

        <div class="nav-section" v-if="!sidebarCollapsed">Заказы и поставки</div>
        <div class="nav-group">
          <a href="#" class="nav-item">
            <i class="pi pi-box" />
            <span v-if="!sidebarCollapsed">Заказы</span>
          </a>
          <a href="#" class="nav-item">
            <i class="pi pi-list" />
            <span v-if="!sidebarCollapsed">Цифровая очередь</span>
          </a>
          <a href="#" class="nav-item">
            <i class="pi pi-truck" />
            <span v-if="!sidebarCollapsed">Самовывоз</span>
          </a>
          <a href="#" class="nav-item">
            <i class="pi pi-map-marker" />
            <span v-if="!sidebarCollapsed">Отслеживание рейсов</span>
          </a>
        </div>

        <div class="nav-section" v-if="!sidebarCollapsed">Тариф</div>
        <div class="nav-group">
          <a href="#" class="nav-item active">
            <i class="pi pi-tag" />
            <span v-if="!sidebarCollapsed">Тендеры</span>
          </a>
          <a href="#" class="nav-item">
            <i class="pi pi-th-large" />
            <span v-if="!sidebarCollapsed">Матрицы</span>
          </a>
        </div>

        <div class="nav-section" v-if="!sidebarCollapsed">Документы и отчёты</div>
        <div class="nav-group">
          <a href="#" class="nav-item">
            <i class="pi pi-check-square" />
            <span v-if="!sidebarCollapsed">Согласования</span>
          </a>
          <a href="#" class="nav-item">
            <i class="pi pi-file" />
            <span v-if="!sidebarCollapsed">Реестры</span>
          </a>
          <a href="#" class="nav-item">
            <i class="pi pi-folder" />
            <span v-if="!sidebarCollapsed">Документооборот</span>
          </a>
        </div>
      </nav>

      <div class="sidebar-footer">
        <div class="user-selector-wrapper" v-if="!sidebarCollapsed">
          <UserSelector compact />
        </div>
        <div class="company-info" v-if="!sidebarCollapsed">
          <div class="company-name">ООО «Контакт Технолоджикс»</div>
          <div class="company-email">kuleshov@contact-tech.ru</div>
        </div>
      </div>
    </aside>

    <!-- Main Area -->
    <main class="main-area">
      <slot />
    </main>

    <!-- Right Panel (for inline mode) -->
    <aside class="right-panel" v-if="$slots.rightPanel">
      <slot name="rightPanel" />
    </aside>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import UserSelector from './UserSelector.vue'
import { useSettings } from '../composables'

defineProps<{
  showBanner?: boolean
}>()

const { settings, updateSettings } = useSettings()

const currentTheme = computed(() => settings.value.theme)

function toggleTheme() {
  updateSettings({ theme: currentTheme.value === 'dark' ? 'light' : 'dark' })
}

defineEmits<{
  (e: 'close-banner'): void
}>()

const sidebarCollapsed = ref(false)
</script>

<style scoped>
/* Theme variables */
.tms-layout--dark {
  --tms-bg: #1f2937;
  --tms-bg-sidebar: #111827;
  --tms-bg-panel: #111827;
  --tms-bg-hover: #374151;
  --tms-text: #f8fafc;
  --tms-text-secondary: #94a3b8;
  --tms-text-muted: #64748b;
  --tms-border: #374151;
  --tms-primary: #60a5fa;
  --tms-primary-bg: rgba(96, 165, 250, 0.15);
  --tms-primary-bg-hover: rgba(96, 165, 250, 0.25);
}

.tms-layout--light {
  --tms-bg: #f8fafc;
  --tms-bg-sidebar: #ffffff;
  --tms-bg-panel: #ffffff;
  --tms-bg-hover: #f1f5f9;
  --tms-text: #334155;
  --tms-text-secondary: #64748b;
  --tms-text-muted: #94a3b8;
  --tms-border: #e2e8f0;
  --tms-primary: #3B82F6;
  --tms-primary-bg: rgba(59, 130, 246, 0.1);
  --tms-primary-bg-hover: rgba(59, 130, 246, 0.2);
}

.tms-layout {
  display: grid;
  grid-template-columns: 220px 1fr;
  grid-template-rows: auto 1fr;
  height: 100vh;
  background: var(--tms-bg);
  color: var(--tms-text);
}

.tms-layout.sidebar-collapsed {
  grid-template-columns: 60px 1fr;
}

.tms-layout.no-banner {
  grid-template-rows: 1fr;
}

.tms-layout.no-banner .sidebar {
  grid-row: 1;
}

.tms-layout.no-banner .main-area {
  grid-row: 1;
}

.tms-layout.no-banner .right-panel {
  grid-row: 1;
}

/* Top Banner */
.top-banner {
  grid-column: 1 / -1;
  background: linear-gradient(135deg, #a8d4f5 0%, #7ec8e3 100%);
  color: #1a1a2e;
  padding: 8px 16px;
  display: flex;
  align-items: center;
  gap: 16px;
}

.banner-content {
  font-weight: 500;
}

.banner-actions {
  display: flex;
  gap: 8px;
  margin-left: auto;
}

.banner-btn {
  padding: 4px 12px;
  border-radius: 4px;
  border: 1px solid #1a1a2e;
  background: transparent;
  color: #1a1a2e;
  font-size: 13px;
  cursor: pointer;
}

.banner-btn.primary {
  background: #1a1a2e;
  color: white;
  border-color: #1a1a2e;
}

.banner-close {
  background: none;
  border: none;
  color: #1a1a2e;
  cursor: pointer;
  padding: 4px;
  opacity: 0.7;
}

.banner-close:hover {
  opacity: 1;
}

/* Sidebar */
.sidebar {
  background: var(--tms-bg-sidebar);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  min-height: 0;
}

.sidebar-header {
  padding: 16px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  border-bottom: 1px solid var(--tms-border);
}

.logo {
  display: flex;
  align-items: center;
  gap: 8px;
  text-decoration: none;
}

.logo-icon {
  font-size: 20px;
}

.logo-text {
  font-weight: 700;
  font-size: 14px;
  color: var(--tms-text);
}

.sidebar-toggle {
  background: none;
  border: none;
  color: var(--tms-text-secondary);
  cursor: pointer;
  padding: 4px;
}

.sidebar-toggle:hover {
  color: var(--tms-text);
}

.sidebar-nav {
  flex: 1;
  overflow-y: auto;
  padding: 8px 0;
}

.nav-section {
  font-size: 11px;
  font-weight: 600;
  color: var(--tms-text-muted);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  padding: 16px 16px 8px;
}

.nav-group {
  margin-bottom: 4px;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 16px;
  color: var(--tms-text-secondary);
  text-decoration: none;
  font-size: 14px;
  transition: all 0.2s;
}

.nav-item:hover {
  background: var(--tms-bg-hover);
  color: var(--tms-text);
}

.nav-item.active {
  color: var(--tms-primary);
  background: var(--tms-primary-bg);
}

.nav-item i {
  font-size: 16px;
  width: 20px;
  text-align: center;
}

.nav-item .badge {
  margin-left: auto;
  background: #e74c3c;
  color: white;
  font-size: 11px;
  padding: 2px 6px;
  border-radius: 10px;
}

.sidebar-footer {
  padding: 16px;
  border-top: 1px solid var(--tms-border);
}

.user-selector-wrapper {
  margin-bottom: 12px;
}

.user-selector-wrapper :deep(.p-select) {
  width: 100%;
  background: var(--tms-bg-hover);
  border-color: var(--tms-border);
}

.user-selector-wrapper :deep(.p-select .p-select-label) {
  color: var(--tms-text);
  font-size: 13px;
}

.company-info {
  font-size: 12px;
}

.company-name {
  color: var(--tms-text);
  font-weight: 500;
  margin-bottom: 2px;
}

.company-email {
  color: var(--tms-text-muted);
}

/* Demo Navigation */
.demo-nav {
  padding: 12px;
  margin-bottom: 8px;
}

.demo-nav-label {
  font-size: 10px;
  font-weight: 700;
  color: var(--tms-primary);
  text-transform: uppercase;
  letter-spacing: 1px;
  padding: 0 4px 8px;
}

.demo-nav-link {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 12px;
  color: var(--tms-text-secondary);
  text-decoration: none;
  font-size: 13px;
  border-radius: 6px;
  transition: all 0.2s;
}

.demo-nav-link:hover {
  background: var(--tms-primary-bg);
  color: var(--tms-text);
}

.demo-nav-link.active {
  background: var(--tms-primary-bg-hover);
  color: var(--tms-primary);
}

.demo-nav-link i {
  font-size: 14px;
  width: 18px;
  text-align: center;
}

.demo-nav-link.theme-toggle {
  width: 100%;
  border: none;
  background: transparent;
  cursor: pointer;
  margin-top: 8px;
  border-top: 1px solid var(--tms-border);
  padding-top: 12px;
}

.demo-nav-collapsed {
  padding: 12px 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
}

.demo-nav-icon {
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--tms-text-secondary);
  text-decoration: none;
  border-radius: 6px;
  transition: all 0.2s;
}

.demo-nav-icon:hover {
  background: var(--tms-primary-bg);
  color: var(--tms-text);
}

.demo-nav-icon.active {
  background: var(--tms-primary-bg-hover);
  color: var(--tms-primary);
}

.demo-nav-icon.theme-toggle {
  border: none;
  cursor: pointer;
  margin-top: 8px;
}

.sidebar-divider {
  height: 1px;
  background: var(--tms-border);
  margin: 0 16px 8px;
}

/* Main Area */
.main-area {
  background: var(--tms-bg);
  overflow: hidden;
  display: flex;
  flex-direction: column;
  min-height: 0;
}

/* Right Panel */
.right-panel {
  width: 380px;
  background: var(--tms-bg-panel);
  border-left: 1px solid var(--tms-border);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  min-height: 0;
}

/* Grid adjustments for right panel */
.tms-layout:has(.right-panel) {
  grid-template-columns: 220px 1fr 380px;
}

.tms-layout.sidebar-collapsed:has(.right-panel) {
  grid-template-columns: 60px 1fr 380px;
}
</style>
