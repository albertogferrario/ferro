import { Head, router, useForm, usePage } from '@inertiajs/react'
import Layout from '../Layout'

export default function Settings() {
  const { visible } = usePage().props as unknown as { visible: boolean }
  const { data, setData } = useForm({ visible })

  function toggle() {
    const next = !data.visible
    setData('visible', next)
    router.post('/settings', { visible: next }, { preserveScroll: true })
  }

  return (
    <Layout active="account">
      <Head title="Impostazioni — Nearly" />
      <h1 className="h1">Impostazioni</h1>
      <p className="sub">Gestisci la tua presenza su Nearly.</p>

      <div className="card">
        <div className="toggle">
          <div>
            <div className="row__title">Visibile sulla mappa</div>
            <div className="row__sub">Se disattivato, gli altri non ti vedranno tra i pin.</div>
          </div>
          <label className="switch">
            <input type="checkbox" checked={data.visible} onChange={toggle} />
            <span className="slider" />
          </label>
        </div>
      </div>

      <div className="card" style={{ marginTop: 14 }}>
        <div className="row__title">Informazioni</div>
        <p className="muted" style={{ fontSize: 14, marginTop: 6, marginBottom: 0 }}>
          Nearly — incontra le persone intorno a te, di persona. Nessuna chat, solo un trillo.
        </p>
      </div>

      <button className="btn btn--danger" style={{ marginTop: 14 }} onClick={() => router.post('/logout')}>
        Esci
      </button>
    </Layout>
  )
}
