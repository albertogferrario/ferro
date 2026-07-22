import { Head, router, usePage } from '@inertiajs/react'
import Layout from '../Layout'

type Trillo = {
  id: number
  from: string
  status: string
  status_label: string
  pending: boolean
}

export default function Trilli() {
  const { trilli } = usePage().props as unknown as { trilli: Trillo[] }

  return (
    <Layout active="trilli">
      <Head title="Trilli — Nearly" />
      <h1 className="h1">I tuoi trilli</h1>
      <p className="sub">Qualcuno ti ha notato. Rispondi… di persona.</p>

      {trilli.length === 0 && (
        <div className="empty">Ancora nessun trillo. Fatti trovare sulla mappa!</div>
      )}

      {trilli.map((t) => (
        <div className="row" key={t.id}>
          <div className="avatar">{t.from.charAt(0)}</div>
          <div className="row__main">
            <div className="row__title">{t.from}</div>
            <div className="row__sub">ti ha inviato un trillo</div>
          </div>
          {t.pending ? (
            <div className="pillrow">
              <button className="btn btn--primary" onClick={() => router.post(`/trilli/${t.id}/accept`)}>
                Accetta
              </button>
              <button className="btn btn--ghost" onClick={() => router.post(`/trilli/${t.id}/decline`)}>
                Ignora
              </button>
            </div>
          ) : (
            <span className={'badge ' + (t.status === 'accepted' ? 'badge--ok' : 'badge--muted')}>
              {t.status_label}
            </span>
          )}
        </div>
      ))}
    </Layout>
  )
}
