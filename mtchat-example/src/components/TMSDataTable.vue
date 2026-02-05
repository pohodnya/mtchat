<template>
  <div class="tms-data-table">
    <!-- Page Header -->
    <div class="page-header">
      <div class="page-title">
        <i class="pi pi-box" />
        <span>Объекты</span>
        <i class="pi pi-info-circle info-icon" />
      </div>
      <div class="page-actions">
        <router-link to="/admin" class="action-btn secondary">
          <i class="pi pi-plus" /> Создать объект
        </router-link>
      </div>
    </div>

    <!-- Tabs by type -->
    <div class="table-tabs">
      <button
        class="tab-btn"
        :class="{ active: activeTab === 'all' }"
        @click="activeTab = 'all'"
      >
        Все ({{ objects.length }})
      </button>
      <button
        v-for="type in objectTypes"
        :key="type"
        class="tab-btn"
        :class="{ active: activeTab === type }"
        @click="activeTab = type"
      >
        {{ typeLabels[type] || type }} ({{ getCountByType(type) }})
      </button>
    </div>

    <!-- Toolbar -->
    <div class="table-toolbar">
      <div class="toolbar-spacer" />
      <div class="search-box">
        <i class="pi pi-search" />
        <input v-model="searchQuery" type="text" placeholder="Поиск" />
      </div>
    </div>

    <!-- Empty State -->
    <div v-if="filteredObjects.length === 0" class="empty-state">
      <i class="pi pi-inbox" />
      <p v-if="objects.length === 0">Нет объектов</p>
      <p v-else>Ничего не найдено</p>
      <router-link v-if="objects.length === 0" to="/admin" class="create-link">
        Создать в Admin Panel
      </router-link>
    </div>

    <!-- Table -->
    <div v-else class="table-container">
      <table class="data-table">
        <thead>
          <tr>
            <th>ID</th>
            <th>Название</th>
            <th>Тип</th>
            <th>Описание</th>
            <th>Создан</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="obj in filteredObjects"
            :key="obj.id"
            :class="{ selected: selectedId === obj.id }"
            @click="$emit('row-click', obj)"
          >
            <td>
              <span class="cell-id">{{ obj.id.slice(0, 8) }}...</span>
            </td>
            <td>
              <span class="cell-title">{{ obj.title }}</span>
            </td>
            <td>
              <span class="cell-type" :class="obj.type">{{ typeLabels[obj.type] || obj.type }}</span>
            </td>
            <td>
              <span class="cell-desc">{{ obj.description || '—' }}</span>
            </td>
            <td>
              <span class="cell-date">{{ formatDate(obj.createdAt) }}</span>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- Pagination -->
    <div v-if="filteredObjects.length > 0" class="table-pagination">
      <div class="pagination-stats">
        {{ filteredObjects.length }} из {{ objects.length }} объектов
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import type { MockObject } from '../types'

const props = defineProps<{
  objects: MockObject[]
  selectedId?: string | null
}>()

defineEmits<{
  (e: 'row-click', obj: MockObject): void
}>()

const activeTab = ref('all')
const searchQuery = ref('')

const typeLabels: Record<string, string> = {
  tender: 'Тендер',
  order: 'Заказ',
  route: 'Рейс',
}

const objectTypes = computed(() => {
  const types = new Set(props.objects.map(o => o.type))
  return Array.from(types)
})

function getCountByType(type: string): number {
  return props.objects.filter(o => o.type === type).length
}

const filteredObjects = computed(() => {
  let result = props.objects

  // Filter by type
  if (activeTab.value !== 'all') {
    result = result.filter(o => o.type === activeTab.value)
  }

  // Filter by search query
  if (searchQuery.value.trim()) {
    const query = searchQuery.value.toLowerCase()
    result = result.filter(o =>
      o.title.toLowerCase().includes(query) ||
      o.description?.toLowerCase().includes(query) ||
      o.id.toLowerCase().includes(query)
    )
  }

  return result
})

function formatDate(isoDate: string): string {
  const date = new Date(isoDate)
  return date.toLocaleDateString('ru-RU', {
    day: '2-digit',
    month: '2-digit',
    year: 'numeric',
  })
}
</script>

<style scoped>
.tms-data-table {
  display: flex;
  flex-direction: column;
  height: 100%;
  color: var(--tms-text, #e0e0e0);
}

/* Page Header */
.page-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 20px;
  border-bottom: 1px solid var(--tms-border, rgba(255, 255, 255, 0.1));
}

.page-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 14px;
}

.page-title i {
  color: var(--tms-text-secondary, #888);
}

.info-icon {
  font-size: 12px;
}

.page-actions {
  display: flex;
  gap: 8px;
}

.action-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  border-radius: 4px;
  border: none;
  font-size: 13px;
  cursor: pointer;
  text-decoration: none;
}

.action-btn.secondary {
  background: transparent;
  border: 1px solid var(--tms-border, #444);
  color: var(--tms-text, #e0e0e0);
}

.action-btn.secondary:hover {
  border-color: var(--tms-primary, #4fc3f7);
  color: var(--tms-primary, #4fc3f7);
}

/* Tabs */
.table-tabs {
  display: flex;
  gap: 0;
  padding: 0 20px;
  border-bottom: 1px solid var(--tms-border, rgba(255, 255, 255, 0.1));
}

.tab-btn {
  padding: 12px 16px;
  background: none;
  border: none;
  color: var(--tms-text-secondary, #888);
  font-size: 14px;
  cursor: pointer;
  border-bottom: 2px solid transparent;
  margin-bottom: -1px;
}

.tab-btn.active {
  color: var(--tms-primary, #4fc3f7);
  border-bottom-color: var(--tms-primary, #4fc3f7);
}

/* Toolbar */
.table-toolbar {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 20px;
  border-bottom: 1px solid var(--tms-border, rgba(255, 255, 255, 0.1));
}

.toolbar-spacer {
  flex: 1;
}

.search-box {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 12px;
  border-radius: 4px;
  border: 1px solid var(--tms-border, #444);
  background: var(--tms-bg-hover, rgba(255, 255, 255, 0.05));
}

.search-box i {
  color: var(--tms-text-muted, #666);
}

.search-box input {
  border: none;
  background: none;
  color: var(--tms-text, #e0e0e0);
  font-size: 13px;
  outline: none;
  width: 200px;
}

.search-box input::placeholder {
  color: var(--tms-text-muted, #666);
}

/* Empty State */
.empty-state {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  color: var(--tms-text-muted, #666);
  gap: 12px;
}

.empty-state i {
  font-size: 48px;
  color: var(--tms-text-muted, #444);
}

.empty-state p {
  font-size: 16px;
  margin: 0;
}

.create-link {
  color: var(--tms-primary, #4fc3f7);
  text-decoration: none;
  font-size: 14px;
}

.create-link:hover {
  text-decoration: underline;
}

/* Table */
.table-container {
  flex: 1;
  overflow: auto;
}

.data-table {
  width: 100%;
  border-collapse: collapse;
}

.data-table th {
  text-align: left;
  padding: 12px 16px;
  font-size: 12px;
  font-weight: 600;
  color: var(--tms-text-secondary, #888);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  border-bottom: 1px solid var(--tms-border, rgba(255, 255, 255, 0.1));
  background: var(--tms-bg-hover, rgba(255, 255, 255, 0.02));
  white-space: nowrap;
}

.data-table td {
  padding: 12px 16px;
  font-size: 14px;
  border-bottom: 1px solid var(--tms-border, rgba(255, 255, 255, 0.05));
}

.data-table tbody tr {
  cursor: pointer;
  transition: background 0.15s;
}

.data-table tbody tr:hover {
  background: var(--tms-bg-hover, rgba(255, 255, 255, 0.03));
}

.data-table tbody tr.selected {
  background: var(--tms-primary-bg, rgba(79, 195, 247, 0.1));
}

.data-table tbody tr.selected:hover {
  background: var(--tms-primary-bg-hover, rgba(79, 195, 247, 0.15));
}

/* Cell styles */
.cell-id {
  font-family: monospace;
  font-size: 12px;
  color: var(--tms-text-secondary, #888);
}

.cell-title {
  font-weight: 500;
  color: var(--tms-text, #e0e0e0);
}

.cell-type {
  display: inline-block;
  padding: 2px 8px;
  border-radius: 4px;
  font-size: 12px;
  font-weight: 500;
}

.cell-type.tender {
  background: rgba(79, 195, 247, 0.2);
  color: #4fc3f7;
}

.cell-type.order {
  background: rgba(129, 199, 132, 0.2);
  color: #81c784;
}

.cell-type.route {
  background: rgba(255, 183, 77, 0.2);
  color: #ffb74d;
}

.cell-desc {
  color: var(--tms-text-secondary, #888);
  max-width: 300px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  display: block;
}

.cell-date {
  color: var(--tms-text-secondary, #888);
  font-size: 13px;
}

/* Pagination */
.table-pagination {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 12px 20px;
  border-top: 1px solid var(--tms-border, rgba(255, 255, 255, 0.1));
  font-size: 13px;
  color: var(--tms-text-secondary, #888);
}

.pagination-stats {
  flex: 1;
}
</style>
