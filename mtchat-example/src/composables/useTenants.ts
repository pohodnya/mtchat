import { computed } from 'vue'
import { useStorage, generateUUID } from './useStorage'
import { STORAGE_KEYS, type Tenant } from '../types'

// Singleton state
const tenants = useStorage<Tenant[]>(STORAGE_KEYS.TENANTS, [])

/**
 * Composable for managing tenants in localStorage
 */
export function useTenants() {

  const sortedTenants = computed(() =>
    [...tenants.value].sort((a, b) => a.name.localeCompare(b.name))
  )

  function createTenant(name: string): Tenant {
    const tenant: Tenant = {
      id: generateUUID(),
      name: name.trim(),
      createdAt: new Date().toISOString(),
    }
    tenants.value = [...tenants.value, tenant]
    return tenant
  }

  function updateTenant(id: string, updates: Partial<Pick<Tenant, 'name'>>): Tenant | null {
    const index = tenants.value.findIndex((t) => t.id === id)
    if (index === -1) return null

    const updated = { ...tenants.value[index], ...updates }
    tenants.value = [
      ...tenants.value.slice(0, index),
      updated,
      ...tenants.value.slice(index + 1),
    ]
    return updated
  }

  function deleteTenant(id: string): boolean {
    const index = tenants.value.findIndex((t) => t.id === id)
    if (index === -1) return false

    tenants.value = [...tenants.value.slice(0, index), ...tenants.value.slice(index + 1)]
    return true
  }

  function getTenant(id: string): Tenant | undefined {
    return tenants.value.find((t) => t.id === id)
  }

  function clearTenants(): void {
    tenants.value = []
  }

  return {
    tenants,
    sortedTenants,
    createTenant,
    updateTenant,
    deleteTenant,
    getTenant,
    clearTenants,
  }
}
