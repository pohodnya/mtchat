import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import dts from 'vite-plugin-dts'
import cssInjectedByJsPlugin from 'vite-plugin-css-injected-by-js'
import { resolve } from 'path'
import { copyFileSync, mkdirSync } from 'fs'

// Plugin to copy theme CSS to dist
function copyThemePlugin() {
  return {
    name: 'copy-theme',
    closeBundle() {
      mkdirSync('dist/theme', { recursive: true })
      copyFileSync('src/theme/aura.css', 'dist/theme/aura.css')
    },
  }
}

export default defineConfig({
  plugins: [
    vue(),
    dts({
      insertTypesEntry: true,
      outDir: 'dist',
      include: ['src/**/*.ts', 'src/**/*.vue'],
    }),
    cssInjectedByJsPlugin(),
    copyThemePlugin(),
  ],
  build: {
    lib: {
      entry: resolve(__dirname, 'src/index.ts'),
      name: 'MTChatVuePrimeVue',
      fileName: 'mtchat-vue-primevue',
      formats: ['es', 'umd'],
    },
    rollupOptions: {
      // Note: @mtchat/vue is bundled (not external) so users only need this single package
      external: ['vue', 'primevue/button', 'primevue/dialog', 'primevue/menu', 'primevue/contextmenu', 'primevue/inputtext', 'primevue/checkbox', 'primevue/radiobutton', 'primevue/tabs', 'primevue/tab', 'primevue/tablist', 'primevue/accordion', 'primevue/accordionpanel', 'primevue/accordionheader', 'primevue/accordioncontent', 'primevue/tooltip'],
      output: {
        globals: {
          vue: 'Vue',
          'primevue/button': 'PrimeVueButton',
          'primevue/dialog': 'PrimeVueDialog',
          'primevue/menu': 'PrimeVueMenu',
          'primevue/contextmenu': 'PrimeVueContextMenu',
          'primevue/inputtext': 'PrimeVueInputText',
          'primevue/checkbox': 'PrimeVueCheckbox',
          'primevue/radiobutton': 'PrimeVueRadioButton',
          'primevue/tabs': 'PrimeVueTabs',
          'primevue/tab': 'PrimeVueTab',
          'primevue/tablist': 'PrimeVueTabList',
          'primevue/accordion': 'PrimeVueAccordion',
          'primevue/accordionpanel': 'PrimeVueAccordionPanel',
          'primevue/accordionheader': 'PrimeVueAccordionHeader',
          'primevue/accordioncontent': 'PrimeVueAccordionContent',
        },
      },
    },
  },
})
