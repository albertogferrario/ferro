import { Head, Link, usePage } from '@inertiajs/react'

export default function Error() {
  const { status, title, message } = usePage().props as unknown as {
    status: number
    title: string
    message: string
  }

  return (
    <div className="splash">
      <Head title={`${status} — Nearly`} />
      <div>
        <div style={{ fontSize: 64, fontWeight: 800, color: 'var(--brand)', lineHeight: 1 }}>
          {status}
        </div>
        <h1 className="h1" style={{ marginTop: 8 }}>
          {title}
        </h1>
        <p className="sub">{message}</p>
      </div>
      <Link href="/map" className="btn btn--primary">
        Torna alla mappa
      </Link>
    </div>
  )
}
