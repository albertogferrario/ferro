import { createInertiaApp } from '@inertiajs/react'
import { createRoot } from 'react-dom/client'
import axios from 'axios'
import 'leaflet/dist/leaflet.css'
import './styles.css'

// CSRF: send the session token (emitted in <meta name="csrf-token">) as the
// X-CSRF-TOKEN header on every request. Inertia uses axios under the hood and
// respects these defaults, so all POST/PUT/PATCH/DELETE are protected.
const csrf = document.querySelector('meta[name="csrf-token"]')?.getAttribute('content')
if (csrf) {
  axios.defaults.headers.common['X-CSRF-TOKEN'] = csrf
}

createInertiaApp({
  resolve: (name) => {
    const pages = import.meta.glob('./pages/**/*.tsx', { eager: true })
    const page = pages[`./pages/${name}.tsx`]
    if (!page) throw new Error(`Unknown Inertia page: ${name}`)
    return page as any
  },
  setup({ el, App, props }) {
    createRoot(el).render(<App {...props} />)
  },
})
