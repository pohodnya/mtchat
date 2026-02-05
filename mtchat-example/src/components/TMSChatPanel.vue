<template>
  <div class="tms-chat-panel">
    <!-- Panel Header -->
    <div class="panel-header">
      <div class="panel-title-row">
        <h3 class="panel-title">
          {{ title }}
          <i class="pi pi-external-link" />
        </h3>
        <span class="panel-tag" v-if="tag">{{ tag }}</span>
      </div>
      <p class="panel-subtitle" v-if="subtitle">{{ subtitle }}</p>
      <div class="panel-actions">
        <button class="panel-action-btn">
          <i class="pi pi-ellipsis-v" />
        </button>
        <button class="panel-action-btn" @click="$emit('close')">
          <i class="pi pi-times" />
        </button>
      </div>
    </div>

    <!-- Panel Tabs -->
    <div class="panel-tabs">
      <button
        v-for="tab in tabs"
        :key="tab"
        class="panel-tab"
        :class="{ active: activeTab === tab }"
        @click="activeTab = tab"
      >
        {{ tab }}
      </button>
    </div>

    <!-- Panel Content -->
    <div class="panel-content">
      <slot />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'

const props = withDefaults(defineProps<{
  title?: string
  subtitle?: string
  tag?: string
  tabs?: string[]
}>(), {
  title: 'Heading',
  subtitle: 'Sub info',
  tag: 'Tag',
  tabs: () => ['Чат', 'Участники', 'Файлы', 'История'],
})

defineEmits<{
  (e: 'close'): void
}>()

const activeTab = ref(props.tabs[0])
</script>

<style scoped>
.tms-chat-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
  background: var(--tms-bg-panel, #16213e);
}

/* Panel Header */
.panel-header {
  padding: 16px;
  border-bottom: 1px solid var(--tms-border, rgba(255, 255, 255, 0.1));
  position: relative;
}

.panel-title-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.panel-title {
  font-size: 16px;
  font-weight: 600;
  color: var(--tms-text, #fff);
  margin: 0;
  display: flex;
  align-items: center;
  gap: 6px;
}

.panel-title i {
  font-size: 12px;
  color: var(--tms-text-muted, #666);
}

.panel-tag {
  padding: 2px 8px;
  border-radius: 4px;
  background: var(--tms-primary, #4fc3f7);
  color: var(--tms-bg, #1a1a2e);
  font-size: 11px;
  font-weight: 600;
}

.panel-subtitle {
  font-size: 13px;
  color: var(--tms-text-secondary, #888);
  margin: 4px 0 0;
}

.panel-actions {
  position: absolute;
  top: 12px;
  right: 12px;
  display: flex;
  gap: 4px;
}

.panel-action-btn {
  width: 28px;
  height: 28px;
  border-radius: 4px;
  border: none;
  background: transparent;
  color: var(--tms-text-secondary, #888);
  cursor: pointer;
}

.panel-action-btn:hover {
  background: var(--tms-bg-hover, rgba(255, 255, 255, 0.1));
  color: var(--tms-text, #fff);
}

/* Panel Tabs */
.panel-tabs {
  display: flex;
  padding: 0 16px;
  border-bottom: 1px solid var(--tms-border, rgba(255, 255, 255, 0.1));
}

.panel-tab {
  padding: 10px 12px;
  background: none;
  border: none;
  color: var(--tms-text-secondary, #888);
  font-size: 13px;
  cursor: pointer;
  border-bottom: 2px solid transparent;
  margin-bottom: -1px;
  white-space: nowrap;
}

.panel-tab.active {
  color: var(--tms-primary, #4fc3f7);
  border-bottom-color: var(--tms-primary, #4fc3f7);
}

/* Panel Content */
.panel-content {
  flex: 1;
  min-height: 0;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}
</style>
