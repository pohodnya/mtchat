<script setup lang="ts">
/**
 * MentionList - Dropdown for @mention suggestions
 */

import { ref, watch } from 'vue'
import type { DialogParticipant } from '../../types'

const props = defineProps<{
  items: DialogParticipant[]
}>()

const emit = defineEmits<{
  select: [participant: DialogParticipant]
}>()

const selectedIndex = ref(0)

// Reset selection when items change
watch(() => props.items, () => {
  selectedIndex.value = 0
})

// Navigation methods (called from parent via ref)
const moveUp = () => {
  if (selectedIndex.value > 0) {
    selectedIndex.value--
  } else {
    selectedIndex.value = props.items.length - 1
  }
}

const moveDown = () => {
  if (selectedIndex.value < props.items.length - 1) {
    selectedIndex.value++
  } else {
    selectedIndex.value = 0
  }
}

const select = () => {
  const item = props.items[selectedIndex.value]
  if (item) {
    emit('select', item)
  }
}

defineExpose({
  moveUp,
  moveDown,
  select,
})
</script>

<template>
  <div class="mtchat-mention-list">
    <div
      v-for="(item, index) in items"
      :key="item.user_id"
      class="mtchat-mention-list__item"
      :class="{ 'mtchat-mention-list__item--selected': index === selectedIndex }"
      @click="emit('select', item)"
      @mouseenter="selectedIndex = index"
    >
      <div class="mtchat-mention-list__avatar">
        {{ (item.display_name || 'U').charAt(0).toUpperCase() }}
      </div>
      <div class="mtchat-mention-list__info">
        <div class="mtchat-mention-list__name">
          {{ item.display_name || 'User' }}
        </div>
        <div v-if="item.company" class="mtchat-mention-list__company">
          {{ item.company }}
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.mtchat-mention-list {
  background: var(--mtchat-bg);
  border: 1px solid var(--mtchat-border);
  border-radius: 8px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.15);
  max-height: 200px;
  width: 250px;
  overflow-y: auto;
  z-index: 1000;
}

.mtchat-mention-list__item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 12px;
  cursor: pointer;
  transition: background 0.1s;
}

.mtchat-mention-list__item:hover,
.mtchat-mention-list__item--selected {
  background: var(--mtchat-bg-hover);
}

.mtchat-mention-list__avatar {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  background: var(--mtchat-primary);
  color: white;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 14px;
  font-weight: 600;
  flex-shrink: 0;
}

.mtchat-mention-list__info {
  flex: 1;
  min-width: 0;
}

.mtchat-mention-list__name {
  font-size: 14px;
  font-weight: 500;
  color: var(--mtchat-text);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.mtchat-mention-list__company {
  font-size: 12px;
  color: var(--mtchat-text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
</style>
