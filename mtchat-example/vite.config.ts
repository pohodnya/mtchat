import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { resolve } from 'path'

export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      '@': resolve(__dirname, 'src'),
      // В dev mode используем исходники SDK напрямую (hot reload)
      '@mtchat/vue': resolve(__dirname, '../mtchat-vue/src/index.ts'),
    },
  },
  // Включаем обработку зависимостей из mtchat-vue
  optimizeDeps: {
    include: [],
    exclude: ['@mtchat/vue', 'pdfjs-dist'],
  },
  server: {
    port: 3000,
    proxy: {
      '/api': {
        target: 'http://localhost:8080',
        changeOrigin: true,
        ws: true,
      },
    },
  },
  build: {
    outDir: 'dist',
  },
})
