import { defineConfig } from 'vitest/config'
import vue from '@vitejs/plugin-vue'

export default defineConfig({ plugins: [vue()], test: { include: ['src/**/*.test.ts'], environment: 'happy-dom', coverage: { provider: 'v8', reporter: ['text', 'lcov'], include: ['src/api.ts', 'src/App.vue'] } } })
