/**
 * MTChat Demo App Types
 *
 * These types are stored in localStorage and used for demo purposes.
 * Real user data comes from the host application in production.
 */

/**
 * Tenant (organization/company)
 * Stored in localStorage for demo
 */
export interface Tenant {
  id: string
  name: string
  createdAt: string
}

/**
 * User with scope configuration
 * Stored in localStorage for demo
 */
export interface User {
  id: string
  name: string
  email?: string
  phone?: string
  tenantId: string
  scopeLevel1: string[] // departments
  scopeLevel2: string[] // permissions/roles
  createdAt: string
}

/**
 * Mock business object (project, task, ticket, etc.)
 * Used for inline mode demonstration.
 * Type can be any string - MTChat does not restrict object types.
 */
export interface MockObject {
  id: string
  type: string
  title: string
  description: string
  createdAt: string
}

/**
 * Access scope for potential participants
 */
export interface AccessScope {
  tenantUid: string
  scopeLevel1: string[]
  scopeLevel2: string[]
}

/**
 * Reference to a dialog created via Management API
 * Stored in localStorage to track which dialogs were created
 */
export interface DialogRef {
  id: string // Real UUID from backend
  objectId: string
  objectType: string
  title?: string
  participants: string[] // User IDs
  accessScopes: AccessScope[]
  createdAt: string
}

/**
 * Supported UI locales
 */
export type Locale = 'ru' | 'en' | 'zh'

/**
 * Application settings
 */
export interface AppSettings {
  adminToken: string
  apiBaseUrl: string
  theme: 'light' | 'dark'
  locale: Locale
}

/**
 * Default settings
 */
export const DEFAULT_SETTINGS: AppSettings = {
  adminToken: '',
  apiBaseUrl: window.location.origin,
  theme: 'light',
  locale: 'ru',
}

/**
 * localStorage keys
 */
export const STORAGE_KEYS = {
  TENANTS: 'mtchat_demo_tenants',
  USERS: 'mtchat_demo_users',
  OBJECTS: 'mtchat_demo_objects',
  DIALOG_REFS: 'mtchat_demo_dialogs',
  CURRENT_USER: 'mtchat_demo_current_user',
  SETTINGS: 'mtchat_demo_settings',
} as const
