import { FormEvent } from 'react'
import { Head, Link, useForm, usePage } from '@inertiajs/react'

export default function Register() {
  const errors = ((usePage().props as any).errors ?? {}) as Record<string, string>
  const { data, setData, post, processing } = useForm({ name: '', email: '', password: '' })

  function submit(e: FormEvent) {
    e.preventDefault()
    post('/register')
  }

  return (
    <div className="splash">
      <Head title="Registrati — Nearly" />
      <div>
        <h1 className="h1">Crea il tuo account</h1>
        <p className="sub">Bastano pochi secondi per iniziare.</p>
      </div>
      <form className="card stack" onSubmit={submit}>
        <div className="field">
          <label>Nome</label>
          <input value={data.name} onChange={(e) => setData('name', e.target.value)} />
          {errors.name && <div className="err">{errors.name}</div>}
        </div>
        <div className="field">
          <label>Email</label>
          <input
            type="email"
            value={data.email}
            onChange={(e) => setData('email', e.target.value)}
            autoComplete="email"
          />
          {errors.email && <div className="err">{errors.email}</div>}
        </div>
        <div className="field">
          <label>Password (min 8 caratteri)</label>
          <input
            type="password"
            value={data.password}
            onChange={(e) => setData('password', e.target.value)}
            autoComplete="new-password"
          />
          {errors.password && <div className="err">{errors.password}</div>}
        </div>
        <button className="btn btn--primary" disabled={processing}>
          Registrati
        </button>
      </form>
      <Link href="/login" className="btn btn--ghost">
        Hai già un account? Accedi
      </Link>
    </div>
  )
}
