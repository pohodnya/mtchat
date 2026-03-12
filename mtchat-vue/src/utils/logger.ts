/**
 * Conditional logger for development mode.
 * Only logs messages when running in development environment.
 */
export const logger = {
  warn: (message: string, ...args: unknown[]) => {
    if (import.meta.env.DEV) {
      console.warn(`[MTChat] ${message}`, ...args)
    }
  },
  error: (message: string, ...args: unknown[]) => {
    if (import.meta.env.DEV) {
      console.error(`[MTChat] ${message}`, ...args)
    }
  },
}
