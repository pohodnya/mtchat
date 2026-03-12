/**
 * JWT Token Generation Utility
 *
 * Generates JWT tokens for demo users when JWT authentication is enabled.
 * Uses the `jose` library for browser-compatible JWT signing.
 */

import * as jose from 'jose'

/**
 * Generate a JWT token for the given user ID
 * @param userId - User ID to include in the token's `sub` claim
 * @param secret - Secret key for HS256 signing
 * @returns JWT token string
 */
export async function generateToken(userId: string, secret: string): Promise<string> {
  const secretKey = new TextEncoder().encode(secret)

  const token = await new jose.SignJWT({ sub: userId })
    .setProtectedHeader({ alg: 'HS256' })
    .setIssuedAt()
    // No expiration - token is reused from host application
    // Backend validates signature only, not expiration
    .sign(secretKey)

  return token
}

/**
 * Cache for generated tokens (userId -> token)
 * Tokens don't expire, so we can cache them for the session
 */
const tokenCache = new Map<string, string>()

/**
 * Get or generate a JWT token for the given user
 * Caches tokens to avoid regenerating on every config computation
 *
 * @param userId - User ID to include in the token
 * @param secret - Secret key for signing
 * @returns JWT token string, or undefined if generation fails
 */
export async function getOrGenerateToken(
  userId: string,
  secret: string
): Promise<string | undefined> {
  // Check cache first (cache key includes secret to handle secret changes)
  const cacheKey = `${userId}:${secret}`
  if (tokenCache.has(cacheKey)) {
    return tokenCache.get(cacheKey)
  }

  try {
    const token = await generateToken(userId, secret)
    tokenCache.set(cacheKey, token)
    return token
  } catch (error) {
    console.error('Failed to generate JWT token:', error)
    return undefined
  }
}

/**
 * Clear the token cache (call when secret changes)
 */
export function clearTokenCache(): void {
  tokenCache.clear()
}
