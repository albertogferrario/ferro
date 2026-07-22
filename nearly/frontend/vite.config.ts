import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

// Build output lands in ../public so the Rust server serves it:
//   - manifest at  public/.vite/manifest.json  (InertiaConfig default)
//   - assets  at   public/assets/*  → served at /assets/* (immutable cache)
export default defineConfig({
  plugins: [react()],
  base: '/',
  server: {
    port: 5173,
    strictPort: true,
    cors: true,
  },
  build: {
    outDir: '../public',
    assetsDir: 'assets',
    emptyOutDir: false,
    manifest: true,
    rollupOptions: {
      input: 'src/main.tsx',
    },
  },
})
