/**
 * Type declarations for vue-virtual-scroller
 * This is an optional peer dependency
 */

declare module 'vue-virtual-scroller' {
  import { DefineComponent } from 'vue'

  export interface DynamicScrollerProps {
    items: any[]
    minItemSize: number
    keyField?: string
    direction?: 'vertical' | 'horizontal'
    listTag?: string
    itemTag?: string
    buffer?: number
    pageMode?: boolean
    prerender?: number
  }

  export interface DynamicScrollerItemProps {
    item: any
    active: boolean
    sizeDependencies?: any[]
    watchData?: boolean
    tag?: string
    emitResize?: boolean
  }

  export const DynamicScroller: DefineComponent<DynamicScrollerProps>
  export const DynamicScrollerItem: DefineComponent<DynamicScrollerItemProps>
  export const RecycleScroller: DefineComponent<any>
}
