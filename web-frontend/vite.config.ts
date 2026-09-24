import { fileURLToPath, URL } from 'node:url'
import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

export default defineConfig({
  plugins: [vue()],
  resolve: { alias: { '@': fileURLToPath(new URL('./src', import.meta.url)) } },
  server: {
    proxy: {
      '/api/v1/login': { target: process.env.IDENTITY_SERVICE_URL ?? 'http://127.0.0.1:8081', rewrite: (path) => path.replace(/^\/api/, '') },
      '/api': { target: process.env.VIDEO_SERVICE_URL ?? 'http://127.0.0.1:8082', rewrite: (path) => path.replace(/^\/api/, '') }
    }
  }
})
