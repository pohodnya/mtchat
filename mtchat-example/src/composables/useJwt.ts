/**
 * JWT Token Composable
 *
 * Provides JWT token generation for the current user.
 * Tokens are generated and cached when JWT auth is enabled.
 */

import { ref, watch, type Ref } from 'vue'
import { useSettings } from './useSettings'
import { useUsers } from './useUsers'
import { getOrGenerateToken, clearTokenCache } from '../utils/jwt'

// Shared token state
const currentToken: Ref<string | undefined> = ref(undefined)
const isGenerating: Ref<boolean> = ref(false)

/**
 * Composable for JWT token generation
 */
export function useJwt() {
  const { settings } = useSettings()
  const { currentUser } = useUsers()

  // Generate token when user or settings change
  watch(
    [() => currentUser.value?.id, () => settings.value.jwtEnabled, () => settings.value.jwtSecret],
    async ([userId, jwtEnabled, jwtSecret]) => {
      // Clear token if JWT disabled or no user
      if (!jwtEnabled || !userId || !jwtSecret) {
        currentToken.value = undefined
        return
      }

      // Generate new token
      isGenerating.value = true
      try {
        currentToken.value = await getOrGenerateToken(userId, jwtSecret)
      } finally {
        isGenerating.value = false
      }
    },
    { immediate: true }
  )

  // Clear cache when secret changes
  watch(
    () => settings.value.jwtSecret,
    () => {
      clearTokenCache()
    }
  )

  return {
    /** Current JWT token (undefined if disabled or not generated) */
    token: currentToken,
    /** Whether token generation is in progress */
    isGenerating,
    /** Whether JWT auth is enabled */
    isEnabled: () => settings.value.jwtEnabled && !!settings.value.jwtSecret,
  }
}
