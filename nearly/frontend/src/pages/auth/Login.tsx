import { FormEvent } from 'react'
import { Head, Link, useForm, usePage } from '@inertiajs/react'

export default function Login() {
  const errors = ((usePage().props as any).errors ?? {}) as Record<string, string>
  const { data, setData, post, processing } = useForm({ email: '', password: '' })

  function submit(e: FormEvent) {
    e.preventDefault()
    post('/login')
  }

  return (
    <div className="splash">
      <Head title="Accedi — Nearly" />
      <div>
        <h1 className="h1">Bentornato</h1>
        <p className="sub">Accedi per continuare su Nearly.</p>
      </div>
      <form className="card stack" onSubmit={submit}>
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
          <label>Password</label>
          <input
            type="password"
            value={data.password}
            onChange={(e) => setData('password', e.target.value)}
            autoComplete="current-password"
          />
        </div>
        <button className="btn btn--primary" disabled={processing}>
          Accedi
        </button>
      </form>
      <Link href="/register" className="btn btn--ghost">
        Non hai un account? Registrati
      </Link>
    </div>
  )
}
