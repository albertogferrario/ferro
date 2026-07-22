import { createInertiaApp } from '@inertiajs/react'
import { createRoot } from 'react-dom/client'
import 'leaflet/dist/leaflet.css'
import './styles.css'

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
