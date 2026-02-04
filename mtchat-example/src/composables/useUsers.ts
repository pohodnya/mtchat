import { computed } from 'vue'
import { useStorage, generateUUID } from './useStorage'
import { STORAGE_KEYS, type User } from '../types'

// Singleton state - shared across all components
const users = useStorage<User[]>(STORAGE_KEYS.USERS, [])
const currentUser = useStorage<User | null>(STORAGE_KEYS.CURRENT_USER, null)

/**
 * Composable for managing users in localStorage
 */
export function useUsers() {

  const sortedUsers = computed(() =>
    [...users.value].sort((a, b) => a.name.localeCompare(b.name))
  )

  function createUser(data: {
    name: string
    tenantId: string
    scopeLevel1?: string[]
    scopeLevel2?: string[]
  }): User {
    const user: User = {
      id: generateUUID(),
      name: data.name.trim(),
      tenantId: data.tenantId,
      scopeLevel1: data.scopeLevel1 || [],
      scopeLevel2: data.scopeLevel2 || [],
      createdAt: new Date().toISOString(),
    }
    users.value = [...users.value, user]
    return user
  }

  function updateUser(
    id: string,
    updates: Partial<Pick<User, 'name' | 'tenantId' | 'scopeLevel1' | 'scopeLevel2'>>
  ): User | null {
    const index = users.value.findIndex((u) => u.id === id)
    if (index === -1) return null

    const updated = { ...users.value[index], ...updates }
    users.value = [
      ...users.value.slice(0, index),
      updated,
      ...users.value.slice(index + 1),
    ]

    // Update current user if it's the one being updated
    if (currentUser.value?.id === id) {
      currentUser.value = updated
    }

    return updated
  }

  function deleteUser(id: string): boolean {
    const index = users.value.findIndex((u) => u.id === id)
    if (index === -1) return false

    users.value = [...users.value.slice(0, index), ...users.value.slice(index + 1)]

    // Clear current user if deleted
    if (currentUser.value?.id === id) {
      currentUser.value = null
    }

    return true
  }

  function getUser(id: string): User | undefined {
    return users.value.find((u) => u.id === id)
  }

  function getUsersByTenant(tenantId: string): User[] {
    return users.value.filter((u) => u.tenantId === tenantId)
  }

  function setCurrentUser(user: User | null): void {
    currentUser.value = user
  }

  function clearUsers(): void {
    users.value = []
    currentUser.value = null
  }

  return {
    users,
    sortedUsers,
    currentUser,
    createUser,
    updateUser,
    deleteUser,
    getUser,
    getUsersByTenant,
    setCurrentUser,
    clearUsers,
  }
}
