import { Head, usePage } from '@inertiajs/react'
import Layout from '../Layout'

type Place = { id: number; name: string; category: string; premium: boolean }

export default function Places() {
  const { places } = usePage().props as unknown as { places: Place[] }

  return (
    <Layout active="places">
      <Head title="Luoghi — Nearly" />
      <h1 className="h1">Luoghi di tendenza</h1>
      <p className="sub">I posti più vivi in città. I premium restano sempre in vista.</p>

      {places.map((p) => (
        <div className="row" key={p.id}>
          <div className="avatar" style={{ background: p.premium ? 'linear-gradient(135deg,#f59e0b,#fbbf24)' : undefined }}>
            {p.premium ? '⭐' : '📍'}
          </div>
          <div className="row__main">
            <div className="row__title">{p.name}</div>
            <div className="row__sub">{p.category}</div>
          </div>
          <span className={'badge ' + (p.premium ? 'badge--premium' : 'badge--muted')}>
            {p.premium ? 'Premium' : 'Trend'}
          </span>
        </div>
      ))}
    </Layout>
  )
}
