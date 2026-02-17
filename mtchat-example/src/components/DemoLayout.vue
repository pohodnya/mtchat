<template>
  <div class="demo-layout" :class="[`demo-layout--${currentTheme}`, { 'sidebar-collapsed': sidebarCollapsed, 'no-banner': !showBanner }]">
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
          <span class="logo-icon">🧬</span>
          <span class="logo-text" v-if="!sidebarCollapsed">ВАШ ЛОГО</span>
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
        <button class="demo-nav-link locale-toggle" @click="cycleLocale">
          <i class="pi pi-globe" />
          {{ localeLabels[currentLocale] }}
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
        <button class="demo-nav-icon locale-toggle" @click="cycleLocale" :title="localeLabels[currentLocale]">
          <i class="pi pi-globe" />
        </button>
      </div>

      <div class="sidebar-footer">
        <div class="user-selector-wrapper" v-if="!sidebarCollapsed">
          <UserSelector compact />
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
import type { Locale } from '../types'

defineProps<{
  showBanner?: boolean
}>()

const { settings, updateSettings } = useSettings()

const currentTheme = computed(() => settings.value.theme)
const currentLocale = computed(() => settings.value.locale)

const localeLabels: Record<Locale, string> = {
  ru: 'Русский',
  en: 'English',
  zh: '中文',
}

const localeOrder: Locale[] = ['ru', 'en', 'zh']

function toggleTheme() {
  updateSettings({ theme: currentTheme.value === 'dark' ? 'light' : 'dark' })
}

function cycleLocale() {
  const currentIndex = localeOrder.indexOf(currentLocale.value)
  const nextIndex = (currentIndex + 1) % localeOrder.length
  updateSettings({ locale: localeOrder[nextIndex] })
}

defineEmits<{
  (e: 'close-banner'): void
}>()

const sidebarCollapsed = ref(false)
</script>

<style scoped>
/* Theme variables */
.demo-layout--dark {
  --demo-bg: #1f2937;
  --demo-bg-sidebar: #111827;
  --demo-bg-panel: #111827;
  --demo-bg-hover: #374151;
  --demo-text: #f8fafc;
  --demo-text-secondary: #94a3b8;
  --demo-text-muted: #64748b;
  --demo-border: #374151;
  --demo-primary: #60a5fa;
  --demo-primary-bg: rgba(96, 165, 250, 0.15);
  --demo-primary-bg-hover: rgba(96, 165, 250, 0.25);
}

.demo-layout--light {
  --demo-bg: #f8fafc;
  --demo-bg-sidebar: #ffffff;
  --demo-bg-panel: #ffffff;
  --demo-bg-hover: #f1f5f9;
  --demo-text: #334155;
  --demo-text-secondary: #64748b;
  --demo-text-muted: #94a3b8;
  --demo-border: #e2e8f0;
  --demo-primary: #3B82F6;
  --demo-primary-bg: rgba(59, 130, 246, 0.1);
  --demo-primary-bg-hover: rgba(59, 130, 246, 0.2);
}

.demo-layout {
  display: grid;
  grid-template-columns: 220px 1fr;
  grid-template-rows: auto 1fr;
  height: 100vh;
  background: var(--demo-bg);
  color: var(--demo-text);
}

.demo-layout.sidebar-collapsed {
  grid-template-columns: 60px 1fr;
}

.demo-layout.no-banner {
  grid-template-rows: 1fr;
}

.demo-layout.no-banner .sidebar {
  grid-row: 1;
}

.demo-layout.no-banner .main-area {
  grid-row: 1;
}

.demo-layout.no-banner .right-panel {
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
  background: var(--demo-bg-sidebar);
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
  border-bottom: 1px solid var(--demo-border);
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
  color: var(--demo-text);
}

.sidebar-toggle {
  background: none;
  border: none;
  color: var(--demo-text-secondary);
  cursor: pointer;
  padding: 4px;
}

.sidebar-toggle:hover {
  color: var(--demo-text);
}

.sidebar-nav {
  flex: 1;
  overflow-y: auto;
  padding: 8px 0;
}

.nav-section {
  font-size: 11px;
  font-weight: 600;
  color: var(--demo-text-muted);
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
  color: var(--demo-text-secondary);
  text-decoration: none;
  font-size: 14px;
  transition: all 0.2s;
}

.nav-item:hover {
  background: var(--demo-bg-hover);
  color: var(--demo-text);
}

.nav-item.active {
  color: var(--demo-primary);
  background: var(--demo-primary-bg);
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
  border-top: 1px solid var(--demo-border);
}

.user-selector-wrapper {
  margin-bottom: 12px;
}

.user-selector-wrapper :deep(.p-select) {
  width: 100%;
  background: var(--demo-bg-hover);
  border-color: var(--demo-border);
}

.user-selector-wrapper :deep(.p-select .p-select-label) {
  color: var(--demo-text);
  font-size: 13px;
}

.company-info {
  font-size: 12px;
}

.company-name {
  color: var(--demo-text);
  font-weight: 500;
  margin-bottom: 2px;
}

.company-email {
  color: var(--demo-text-muted);
}

/* Demo Navigation */
.demo-nav {
  padding: 12px;
  margin-bottom: 8px;
}

.demo-nav-label {
  font-size: 10px;
  font-weight: 700;
  color: var(--demo-primary);
  text-transform: uppercase;
  letter-spacing: 1px;
  padding: 0 4px 8px;
}

.demo-nav-link {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 12px;
  color: var(--demo-text-secondary);
  text-decoration: none;
  font-size: 13px;
  border-radius: 6px;
  transition: all 0.2s;
}

.demo-nav-link:hover {
  background: var(--demo-primary-bg);
  color: var(--demo-text);
}

.demo-nav-link.active {
  background: var(--demo-primary-bg-hover);
  color: var(--demo-primary);
}

.demo-nav-link i {
  font-size: 14px;
  width: 18px;
  text-align: center;
}

.demo-nav-link.theme-toggle,
.demo-nav-link.locale-toggle {
  width: 100%;
  border: none;
  background: transparent;
  cursor: pointer;
  margin-top: 8px;
  border-top: 1px solid var(--demo-border);
  padding-top: 12px;
}

.demo-nav-link.locale-toggle {
  margin-top: 4px;
  border-top: none;
  padding-top: 8px;
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
  color: var(--demo-text-secondary);
  text-decoration: none;
  border-radius: 6px;
  transition: all 0.2s;
}

.demo-nav-icon:hover {
  background: var(--demo-primary-bg);
  color: var(--demo-text);
}

.demo-nav-icon.active {
  background: var(--demo-primary-bg-hover);
  color: var(--demo-primary);
}

.demo-nav-icon.theme-toggle,
.demo-nav-icon.locale-toggle {
  border: none;
  cursor: pointer;
  margin-top: 8px;
}

.demo-nav-icon.locale-toggle {
  margin-top: 4px;
}

.sidebar-divider {
  height: 1px;
  background: var(--demo-border);
  margin: 0 16px 8px;
}

/* Main Area */
.main-area {
  background: var(--demo-bg);
  overflow: hidden;
  display: flex;
  flex-direction: column;
  min-height: 0;
}

/* Right Panel */
.right-panel {
  width: 480px;
  background: var(--demo-bg-panel);
  border-left: 1px solid var(--demo-border);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  min-height: 0;
}

/* Grid adjustments for right panel */
.demo-layout:has(.right-panel) {
  grid-template-columns: 220px 1fr 480px;
}

.demo-layout.sidebar-collapsed:has(.right-panel) {
  grid-template-columns: 60px 1fr 480px;
}
</style>
