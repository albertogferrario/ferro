import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

// Build output must line up with the framework's Inertia asset contract:
//   - manifest → ../public/.vite/manifest.json  (InertiaConfig default manifest_path)
//   - assets   → ../public/assets/*  → served at /assets/* by the static handler
// (Using outDir '../public/assets' nests files under assets/assets/ and puts the
// manifest off the default path, which 404s in production.)
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
