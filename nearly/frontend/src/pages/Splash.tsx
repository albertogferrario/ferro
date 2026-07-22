import { Head, Link } from '@inertiajs/react'

export default function Splash() {
  return (
    <div className="splash">
      <Head title="Nearly — di persona, non in chat" />
      <div className="splash__logo" aria-hidden>
        <svg width="40" height="40" viewBox="0 0 100 100" fill="none">
          <circle cx="22" cy="72" r="12" fill="#fff" />
          <circle cx="78" cy="28" r="12" fill="#fff" />
          <path
            d="M24 46c14-4 20 12 34 8"
            stroke="#fff"
            strokeWidth="14"
            strokeLinecap="round"
            opacity="0.95"
          />
        </svg>
      </div>
      <div>
        <h1>Nearly</h1>
        <p>Scopri chi e cosa ti circonda — di persona, non in chat.</p>
      </div>
      <div className="stack">
        <Link href="/map" className="btn btn--primary">
          Esplora la mappa
        </Link>
        <Link href="/login" className="btn btn--secondary">
          Accedi
        </Link>
        <Link href="/register" className="btn btn--ghost">
          Crea un account
        </Link>
      </div>
    </div>
  )
}
