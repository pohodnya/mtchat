import { ref, watch, type Ref } from 'vue'

/**
 * Generic reactive localStorage wrapper
 * Automatically syncs with localStorage and provides reactivity
 */
export function useStorage<T>(key: string, defaultValue: T): Ref<T> {
  // Read initial value from localStorage
  const storedValue = localStorage.getItem(key)
  let initialValue: T = defaultValue

  if (storedValue !== null) {
    try {
      const parsed = JSON.parse(storedValue)
      // Merge with defaults to handle new fields added to schema
      if (typeof defaultValue === 'object' && defaultValue !== null && !Array.isArray(defaultValue)) {
        initialValue = { ...defaultValue, ...parsed }
      } else {
        initialValue = parsed
      }
    } catch {
      console.warn(`Failed to parse localStorage value for key "${key}"`)
    }
  }

  const data = ref<T>(initialValue) as Ref<T>

  // Watch for changes and sync to localStorage
  watch(
    data,
    (newValue) => {
      if (newValue === null || newValue === undefined) {
        localStorage.removeItem(key)
      } else {
        localStorage.setItem(key, JSON.stringify(newValue))
      }
    },
    { deep: true }
  )

  // Listen for storage events from other tabs
  const handleStorageChange = (event: StorageEvent) => {
    if (event.key === key) {
      if (event.newValue === null) {
        data.value = defaultValue
      } else {
        try {
          data.value = JSON.parse(event.newValue)
        } catch {
          console.warn(`Failed to parse storage event value for key "${key}"`)
        }
      }
    }
  }

  window.addEventListener('storage', handleStorageChange)

  return data
}

/**
 * Generate a UUID v4
 */
export function generateUUID(): string {
  return crypto.randomUUID()
}
