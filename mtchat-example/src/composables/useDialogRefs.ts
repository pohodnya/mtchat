import { computed } from 'vue'
import { useStorage } from './useStorage'
import { STORAGE_KEYS, type DialogRef, type AccessScope } from '../types'

// Singleton state
const dialogRefs = useStorage<DialogRef[]>(STORAGE_KEYS.DIALOG_REFS, [])

/**
 * Composable for managing dialog references in localStorage
 * These are references to real dialogs created via Management API
 */
export function useDialogRefs() {

  const sortedDialogRefs = computed(() =>
    [...dialogRefs.value].sort(
      (a, b) => new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime()
    )
  )

  function addDialogRef(data: {
    id: string // Real UUID from backend
    objectId: string
    objectType: string
    title?: string
    participants: string[]
    accessScopes: AccessScope[]
  }): DialogRef {
    const ref: DialogRef = {
      ...data,
      createdAt: new Date().toISOString(),
    }
    dialogRefs.value = [...dialogRefs.value, ref]
    return ref
  }

  function removeDialogRef(id: string): boolean {
    const index = dialogRefs.value.findIndex((d) => d.id === id)
    if (index === -1) return false

    dialogRefs.value = [
      ...dialogRefs.value.slice(0, index),
      ...dialogRefs.value.slice(index + 1),
    ]
    return true
  }

  function getDialogRef(id: string): DialogRef | undefined {
    return dialogRefs.value.find((d) => d.id === id)
  }

  function getDialogRefByObject(objectType: string, objectId: string): DialogRef | undefined {
    return dialogRefs.value.find(
      (d) => d.objectType === objectType && d.objectId === objectId
    )
  }

  function clearDialogRefs(): void {
    dialogRefs.value = []
  }

  return {
    dialogRefs,
    sortedDialogRefs,
    addDialogRef,
    removeDialogRef,
    getDialogRef,
    getDialogRefByObject,
    clearDialogRefs,
  }
}
