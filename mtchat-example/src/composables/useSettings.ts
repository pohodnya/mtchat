import { useStorage } from './useStorage'
import { STORAGE_KEYS, DEFAULT_SETTINGS, type AppSettings } from '../types'

// Singleton state
const settings = useStorage<AppSettings>(STORAGE_KEYS.SETTINGS, DEFAULT_SETTINGS)

/**
 * Composable for managing app settings in localStorage
 */
export function useSettings() {

  function updateSettings(updates: Partial<AppSettings>): void {
    settings.value = { ...settings.value, ...updates }
  }

  function resetSettings(): void {
    settings.value = { ...DEFAULT_SETTINGS }
  }

  return {
    settings,
    updateSettings,
    resetSettings,
  }
}
