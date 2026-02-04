<template>
  <div class="object-card">
    <div class="card-header">
      <Tag :value="object.type" severity="info" />
      <span class="card-date">{{ formatDate(object.createdAt) }}</span>
    </div>
    <h3 class="card-title">{{ object.title }}</h3>
    <p v-if="object.description" class="card-description">
      {{ object.description }}
    </p>
    <div class="card-footer">
      <code class="object-id">ID: {{ object.id.slice(0, 8) }}...</code>
    </div>
  </div>
</template>

<script setup lang="ts">
import Tag from 'primevue/tag'
import type { MockObject } from '../types'

defineProps<{
  object: MockObject
}>()

function formatDate(dateString: string): string {
  return new Date(dateString).toLocaleDateString()
}
</script>

<style scoped>
.object-card {
  background: white;
  border-radius: 12px;
  padding: 20px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
  height: 100%;
  display: flex;
  flex-direction: column;
}

.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
}

.card-date {
  font-size: 12px;
  color: #999;
}

.card-title {
  font-size: 18px;
  font-weight: 600;
  color: #1a1a1a;
  margin-bottom: 8px;
}

.card-description {
  color: #666;
  font-size: 14px;
  line-height: 1.5;
  flex: 1;
  margin-bottom: 16px;
}

.card-footer {
  padding-top: 12px;
  border-top: 1px solid #f0f0f0;
}

.object-id {
  font-family: 'SF Mono', Monaco, monospace;
  font-size: 11px;
  background: #f5f5f5;
  padding: 4px 8px;
  border-radius: 4px;
  color: #666;
}
</style>
