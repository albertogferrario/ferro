import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import tailwindcss from '@tailwindcss/vite'

export default defineConfig({
  plugins: [react(), tailwindcss()],
  server: {
    port: 5173,
    cors: true,
  },
  build: {
    outDir: '../public',
    manifest: true,
    rollupOptions: {
      input: 'src/main.tsx',
    },
  },
})
