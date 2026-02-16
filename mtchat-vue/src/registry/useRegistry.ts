/**
 * Component Registry - provide/inject hooks
 *
 * Allows swapping UI primitives between native HTML and framework implementations
 *
 * @example Usage in host app
 * ```vue
 * <script setup>
 * import { MTChat, provideRegistry } from '@mtchat/vue'
 * import { primevueRegistry } from '@mtchat/vue-primevue'
 *
 * // Use PrimeVue components
 * provideRegistry(primevueRegistry)
 * </script>
 * ```
 *
 * @example Usage in MTChat components
 * ```vue
 * <script setup>
 * import { useRegistry } from '../registry/useRegistry'
 * const { MtButton, MtDialog, vTooltip } = useRegistry()
 * </script>
 * ```
 */

import { provide, inject, type InjectionKey } from 'vue'
import type { ComponentRegistry, PartialRegistry } from './types'
import { defaultRegistry } from './defaultRegistry'

/**
 * Injection key for component registry
 */
export const REGISTRY_KEY: InjectionKey<ComponentRegistry> = Symbol('mtchat-registry')

/**
 * Provide a component registry to child components
 *
 * Call this in the parent component (e.g., App.vue) before using MTChat
 * to override the default native components with framework-specific ones
 *
 * @param registry - Partial or full registry to use
 */
export function provideRegistry(registry: PartialRegistry): void {
  // Merge with default registry to ensure all components are available
  const mergedRegistry: ComponentRegistry = {
    ...defaultRegistry,
    ...registry,
  }
  provide(REGISTRY_KEY, mergedRegistry)
}

/**
 * Inject the component registry
 *
 * Use this in chat components to get the appropriate UI primitives
 * Falls back to default native components if no registry is provided
 *
 * @returns Component registry with all UI primitives
 */
export function useRegistry(): ComponentRegistry {
  return inject(REGISTRY_KEY, defaultRegistry)
}

/**
 * Get a specific component from the registry
 *
 * Convenience function for getting a single component
 *
 * @param name - Component name from registry
 * @returns The component
 */
export function useRegistryComponent<K extends keyof ComponentRegistry>(
  name: K
): ComponentRegistry[K] {
  const registry = useRegistry()
  return registry[name]
}
