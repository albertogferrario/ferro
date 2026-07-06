import { ReactNode } from 'react'
import { Link } from '@inertiajs/react'
import { Toaster, GlassCard } from '@appolabs/ui'

interface AuthLayoutProps {
  children: ReactNode
  title: string
  subtitle?: string
}

export function AuthLayout({ children, title, subtitle }: AuthLayoutProps) {
  return (
    <div className="relative min-h-screen bg-background">
      <Toaster />

      {/* Decorative gradient orbs */}
      <div className="pointer-events-none fixed inset-0 overflow-hidden">
        <div className="absolute -left-32 -top-32 h-96 w-96 animate-float rounded-full bg-primary/20 blur-3xl" />
        <div className="absolute -right-32 top-1/4 h-80 w-80 animate-float rounded-full bg-primary/15 blur-3xl" style={{ animationDelay: '-2s' }} />
        <div className="absolute -bottom-32 right-1/4 h-72 w-72 animate-float rounded-full bg-accent/20 blur-3xl" style={{ animationDelay: '-4s' }} />
      </div>

      <div className="relative z-10 flex min-h-screen flex-col lg:flex-row">
        {/* Hero section - Left side (desktop only) */}
        <div className="hidden lg:flex lg:w-1/2">
          <div className="mx-auto flex max-w-xl flex-col justify-center p-12 lg:p-16">
            <div className="mb-8">
              <Link href="/" className="flex items-center gap-2 text-primary">
                <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-primary text-primary-foreground">
                  <svg className="h-6 w-6" fill="currentColor" viewBox="0 0 24 24">
                    <path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5" />
                  </svg>
                </div>
                <span className="text-2xl font-bold">{{name}}</span>
              </Link>
            </div>

            <h1 className="mb-6 text-4xl font-bold tracking-tight text-foreground lg:text-5xl">
              Welcome to {{name}}
            </h1>

            <p className="mb-10 text-lg leading-relaxed text-muted-foreground">
              A modern web application built with Ferro framework. Fast, secure, and scalable.
            </p>

            <div className="space-y-3">
              <div className="flex items-center gap-3 text-muted-foreground">
                <div className="flex h-6 w-6 items-center justify-center rounded-full bg-primary text-xs text-primary-foreground">✓</div>
                <span>Built with Rust for maximum performance</span>
              </div>
              <div className="flex items-center gap-3 text-muted-foreground">
                <div className="flex h-6 w-6 items-center justify-center rounded-full bg-primary text-xs text-primary-foreground">✓</div>
                <span>React frontend with TypeScript</span>
              </div>
              <div className="flex items-center gap-3 text-muted-foreground">
                <div className="flex h-6 w-6 items-center justify-center rounded-full bg-primary text-xs text-primary-foreground">✓</div>
                <span>Seamless SPA navigation with Inertia.js</span>
              </div>
            </div>
          </div>
        </div>

        {/* Form section - Right side */}
        <div className="flex flex-1 flex-col lg:w-1/2">
          {/* Mobile header */}
          <header className="flex items-center justify-center px-6 py-4 lg:hidden">
            <Link href="/" className="flex items-center gap-2 text-primary">
              <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-primary text-primary-foreground">
                <svg className="h-5 w-5" fill="currentColor" viewBox="0 0 24 24">
                  <path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5" />
                </svg>
              </div>
              <span className="text-xl font-bold">{{name}}</span>
            </Link>
          </header>

          <div className="flex flex-1 items-center justify-center p-6">
            <div className="w-full max-w-md">
              <div className="mb-8 text-center">
                <h1 className="text-3xl font-bold text-foreground">{title}</h1>
                {subtitle && (
                  <p className="mt-2 text-muted-foreground">{subtitle}</p>
                )}
              </div>

              <GlassCard variant="auth" padding="lg">
                {children}
              </GlassCard>

              {/* Footer */}
              <footer className="mt-6 text-center text-sm text-muted-foreground">
                <p>&copy; {new Date().getFullYear()} {{name}}. All rights reserved.</p>
              </footer>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}
