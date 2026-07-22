import { Head, Link, useForm, usePage } from '@inertiajs/react'
import Layout from '../Layout'

export default function User() {
  const { user_id, display_name, status } = usePage().props as unknown as {
    user_id: number
    display_name: string
    status: string
  }
  const { post, processing } = useForm({ to_user_id: user_id })

  return (
    <Layout active="map">
      <Head title={`${display_name} — Nearly`} />
      <div className="card stack" style={{ textAlign: 'center', alignItems: 'center' }}>
        <div className="avatar" style={{ width: 72, height: 72, fontSize: 28 }}>
          {display_name.charAt(0)}
        </div>
        <div>
          <div className="h1">{display_name}</div>
          <p className="sub" style={{ margin: 0 }}>
            {status}
          </p>
        </div>
        <p className="muted" style={{ fontSize: 13, margin: 0 }}>
          Su Nearly non c'è la chat: manda un trillo e presentati di persona.
        </p>
        <button
          className="btn btn--primary"
          disabled={processing}
          onClick={() => post('/trilli')}
        >
          Invia un trillo 🔔
        </button>
        <Link href="/map" className="btn btn--ghost">
          Torna alla mappa
        </Link>
      </div>
    </Layout>
  )
}
