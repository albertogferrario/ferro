import { Link, usePage, router } from '@inertiajs/react'
import { ReactNode } from 'react'

type Auth = { id: number; name: string; display_name: string } | null

const TABS = [
  { key: 'map', href: '/map', icon: '🗺️', label: 'Mappa' },
  { key: 'trilli', href: '/trilli', icon: '🔔', label: 'Trilli' },
  { key: 'places', href: '/places', icon: '📍', label: 'Luoghi' },
  { key: 'account', href: '/account', icon: '👤', label: 'Account' },
]

export default function Layout({
  active,
  children,
  bare = false,
}: {
  active?: string
  children: ReactNode
  bare?: boolean
}) {
  const auth = (usePage().props as any).auth as Auth

  return (
    <div className="shell">
      <div className="shell__top">
        <Link href="/map" className="wordmark">Nearly</Link>
        {auth ? (
          <button className="linklike" onClick={() => router.post('/logout')}>
            Esci
          </button>
        ) : (
          <Link href="/login" className="linklike">
            Accedi
          </Link>
        )}
      </div>

      <div className={bare ? 'shell__body' : 'shell__body shell__body--pad'}>{children}</div>

      <nav className="tabs">
        {TABS.map((t) => (
          <Link
            key={t.key}
            href={t.href}
            className={'tab' + (active === t.key ? ' tab--active' : '')}
          >
            <span className="tab__icon">{t.icon}</span>
            {t.label}
          </Link>
        ))}
      </nav>
    </div>
  )
}
