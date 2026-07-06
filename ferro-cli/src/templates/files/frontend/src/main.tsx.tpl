import { createInertiaApp } from '@inertiajs/react'
import { createRoot } from 'react-dom/client'
import axios from 'axios'
import './styles/globals.css'

// Configure axios defaults for CSRF protection
axios.defaults.headers.common['X-Requested-With'] = 'XMLHttpRequest'

// Get CSRF token from meta tag and set as default header
const csrfToken = document.querySelector('meta[name="csrf-token"]')?.getAttribute('content')
if (csrfToken) {
  axios.defaults.headers.common['X-CSRF-TOKEN'] = csrfToken
}

createInertiaApp({
  resolve: (name) => {
    // Match both root-level and nested page components
    const pages = import.meta.glob(['./pages/**/*.tsx', '!**/*.test.tsx'], { eager: true })
    return pages[`./pages/${name}.tsx`]
  },
  setup({ el, App, props }) {
    createRoot(el).render(<App {...props} />)
  },
})
