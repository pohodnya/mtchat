import { computed } from 'vue'
import { useStorage, generateUUID } from './useStorage'
import { STORAGE_KEYS, type MockObject } from '../types'

/**
 * Default object types for demo
 */
export const OBJECT_TYPES = ['tender', 'order', 'route'] as const

// Singleton state
const objects = useStorage<MockObject[]>(STORAGE_KEYS.OBJECTS, [])

/**
 * Composable for managing mock business objects in localStorage
 */
export function useObjects() {

  const sortedObjects = computed(() =>
    [...objects.value].sort(
      (a, b) => new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime()
    )
  )

  const objectsByType = computed(() => {
    const grouped: Record<string, MockObject[]> = {}
    for (const obj of objects.value) {
      if (!grouped[obj.type]) {
        grouped[obj.type] = []
      }
      grouped[obj.type].push(obj)
    }
    return grouped
  })

  function createObject(data: {
    type: string
    title: string
    description?: string
  }): MockObject {
    const obj: MockObject = {
      id: generateUUID(),
      type: data.type,
      title: data.title.trim(),
      description: data.description?.trim() || '',
      createdAt: new Date().toISOString(),
    }
    objects.value = [...objects.value, obj]
    return obj
  }

  function updateObject(
    id: string,
    updates: Partial<Pick<MockObject, 'type' | 'title' | 'description'>>
  ): MockObject | null {
    const index = objects.value.findIndex((o) => o.id === id)
    if (index === -1) return null

    const updated = { ...objects.value[index], ...updates }
    objects.value = [
      ...objects.value.slice(0, index),
      updated,
      ...objects.value.slice(index + 1),
    ]
    return updated
  }

  function deleteObject(id: string): boolean {
    const index = objects.value.findIndex((o) => o.id === id)
    if (index === -1) return false

    objects.value = [...objects.value.slice(0, index), ...objects.value.slice(index + 1)]
    return true
  }

  function getObject(id: string): MockObject | undefined {
    return objects.value.find((o) => o.id === id)
  }

  function getObjectsByType(type: string): MockObject[] {
    return objects.value.filter((o) => o.type === type)
  }

  function clearObjects(): void {
    objects.value = []
  }

  return {
    objects,
    sortedObjects,
    objectsByType,
    createObject,
    updateObject,
    deleteObject,
    getObject,
    getObjectsByType,
    clearObjects,
    OBJECT_TYPES,
  }
}
