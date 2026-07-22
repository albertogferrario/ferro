import { FormEvent } from 'react'
import { Head, Link, useForm, usePage } from '@inertiajs/react'
import Layout from '../Layout'

export default function Account() {
  const { display_name, status } = usePage().props as unknown as {
    display_name: string
    status: string
  }
  const { data, setData, post, processing, recentlySuccessful } = useForm({
    display_name,
    status,
  })

  function submit(e: FormEvent) {
    e.preventDefault()
    post('/account')
  }

  return (
    <Layout active="account">
      <Head title="Account — Nearly" />
      <h1 className="h1">Il tuo profilo</h1>
      <p className="sub">Come ti vedono le persone vicine.</p>

      <form className="card stack" onSubmit={submit}>
        <div className="field">
          <label>Nome visualizzato</label>
          <input value={data.display_name} onChange={(e) => setData('display_name', e.target.value)} />
        </div>
        <div className="field">
          <label>Status</label>
          <input value={data.status} onChange={(e) => setData('status', e.target.value)} />
        </div>
        <button className="btn btn--primary" disabled={processing}>
          {recentlySuccessful ? 'Salvato ✓' : 'Salva'}
        </button>
      </form>

      <Link href="/settings" className="btn btn--ghost" style={{ marginTop: 12 }}>
        Impostazioni →
      </Link>
    </Layout>
  )
}
