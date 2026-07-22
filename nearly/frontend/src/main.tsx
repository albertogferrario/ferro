import { createInertiaApp } from '@inertiajs/react'
import { createRoot } from 'react-dom/client'
import axios from 'axios'
import 'leaflet/dist/leaflet.css'
import './styles.css'
import { xsrfToken } from './useChannel'

// CSRF: read the framework's JS-readable XSRF-TOKEN cookie fresh on every request
// and echo it as X-XSRF-TOKEN. Reading per-request (not once from the <meta> tag)
// keeps the token correct even after it rotates on login. Inertia uses this axios
// instance, so all POST/PUT/PATCH/DELETE are protected.
axios.interceptors.request.use((config) => {
  const t = xsrfToken()
  if (t) config.headers['X-XSRF-TOKEN'] = t
  return config
})

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
