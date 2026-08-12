// Projection-native Inertia page (SUBST-03 dogfood).
//
// Nothing here is hand-specialized to "orders": columns come from `schema.fields`,
// rows from the tenant-scoped `data`, and the action bar from `schema.actions`
// filtered by `permitted_actions`. The same component would render any projected
// ServiceDef. All props are produced by `Inertia::from_projection`.

interface FieldContract {
  name: string
  data_type: string
  meaning: string
  readable: boolean
  writable: boolean
}

interface ActionContract {
  name: string
  display_name?: string | null
  preconditions?: string[]
  is_transition: boolean
}

interface SchemaContract {
  name: string
  display_name?: string | null
  fields: FieldContract[]
  actions: ActionContract[]
  guards: string[]
  has_state_machine: boolean
}

interface OrderListProps {
  schema: SchemaContract
  data: Record<string, unknown>[]
  permitted_actions: string[]
  total: number
  limit: number
  offset: number
}

const asStr = (v: unknown): string =>
  typeof v === 'string' ? v : v == null ? '' : JSON.stringify(v)

const titleize = (s: string): string =>
  s.replace(/_/g, ' ').replace(/\b\w/g, (c) => c.toUpperCase())

const STATUS_CLASSES: Record<string, string> = {
  draft: 'bg-muted/15 text-muted',
  submitted: 'bg-accent/15 text-accent',
  approved: 'bg-success/15 text-success',
  shipped: 'bg-success/15 text-success',
  delivered: 'bg-success text-white',
  cancelled: 'bg-destructive/15 text-destructive',
}

function Cell({ field, value }: { field: FieldContract; value: unknown }) {
  const meaning = asStr(field.meaning).toLowerCase()

  if (meaning.includes('money')) {
    const n = typeof value === 'number' ? value : parseFloat(asStr(value))
    return (
      <span className="tabular-nums font-medium">
        {isNaN(n) ? asStr(value) : `$${n.toFixed(2)}`}
      </span>
    )
  }

  if (meaning.includes('status')) {
    const v = asStr(value).toLowerCase()
    return (
      <span
        className={`inline-flex rounded-md px-2 py-0.5 text-xs font-medium ${
          STATUS_CLASSES[v] ?? 'bg-muted/15 text-muted'
        }`}
      >
        {asStr(value)}
      </span>
    )
  }

  if (meaning.includes('identifier')) {
    return <span className="font-mono text-muted">#{asStr(value)}</span>
  }

  if (meaning.includes('date') || meaning.includes('created')) {
    const d = new Date(asStr(value))
    return (
      <span className="tabular-nums text-muted">
        {isNaN(d.getTime()) ? asStr(value) : d.toLocaleDateString()}
      </span>
    )
  }

  return <span>{asStr(value)}</span>
}

export default function OrderList({
  schema,
  data,
  permitted_actions,
  total,
  limit,
  offset,
}: OrderListProps) {
  const permitted = new Set(permitted_actions)
  const columns = schema.fields.filter((f) => f.readable)

  return (
    <div className="min-h-screen bg-background text-foreground">
      <div className="mx-auto max-w-5xl px-6 py-12">
        {/* Header */}
        <header className="mb-8">
          <div className="mb-2 inline-flex items-center gap-2 text-xs font-medium uppercase tracking-wide text-muted">
            <span className="inline-block h-1.5 w-1.5 rounded-full bg-accent" />
            Projection · Inertia
          </div>
          <h1 className="text-3xl font-semibold tracking-tight">
            {schema.display_name ?? titleize(schema.name)}
          </h1>
          <p className="mt-2 max-w-2xl text-sm leading-relaxed text-muted">
            Columns, rows, and the actions below are all derived from one{' '}
            <code className="rounded bg-card px-1 py-0.5 font-mono text-[0.8em] text-foreground ring-1 ring-border">
              ServiceDef
            </code>{' '}
            via{' '}
            <code className="rounded bg-card px-1 py-0.5 font-mono text-[0.8em] text-foreground ring-1 ring-border">
              Inertia::from_projection
            </code>{' '}
            — the same declaration that drives the MCP and visual renderers.
          </p>
        </header>

        {/* Permitted actions — declared actions, dimmed when hidden by a guard */}
        <section className="mb-6">
          <div className="mb-2 text-xs font-medium uppercase tracking-wide text-muted">
            Available actions{' '}
            <span className="normal-case text-muted/70">
              — guard-filtered for your tenant
            </span>
          </div>
          <div className="flex flex-wrap items-center gap-2">
            {schema.actions.map((a) => {
              const allowed = permitted.has(a.name)
              return (
                <span
                  key={a.name}
                  title={
                    allowed
                      ? (a.preconditions ?? []).length
                        ? `guarded by: ${(a.preconditions ?? []).join(', ')}`
                        : 'no preconditions'
                      : `hidden — precondition not met: ${(a.preconditions ?? []).join(', ')}`
                  }
                  className={
                    allowed
                      ? 'inline-flex items-center rounded-md bg-primary px-3 py-1.5 text-sm font-medium text-primary-foreground'
                      : 'inline-flex items-center gap-1 rounded-md px-3 py-1.5 text-sm font-medium text-muted line-through ring-1 ring-border'
                  }
                >
                  {a.display_name ?? titleize(a.name)}
                  {!allowed && <span className="no-underline">🔒</span>}
                </span>
              )
            })}
          </div>
        </section>

        {/* Data table */}
        <div className="overflow-hidden rounded-lg bg-surface ring-1 ring-border">
          <table className="w-full text-left text-sm">
            <thead>
              <tr className="border-b border-border bg-card">
                {columns.map((f) => (
                  <th
                    key={f.name}
                    className="px-4 py-3 font-medium text-muted"
                  >
                    {titleize(f.name)}
                  </th>
                ))}
              </tr>
            </thead>
            <tbody>
              {data.length === 0 ? (
                <tr>
                  <td
                    colSpan={columns.length}
                    className="px-4 py-10 text-center text-muted"
                  >
                    No rows for this tenant.
                  </td>
                </tr>
              ) : (
                data.map((row, i) => (
                  <tr
                    key={i}
                    className="border-b border-border/60 last:border-0 hover:bg-card"
                  >
                    {columns.map((f) => (
                      <td key={f.name} className="px-4 py-3">
                        <Cell field={f} value={row[f.name]} />
                      </td>
                    ))}
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </div>

        {/* Footer meta */}
        <footer className="mt-4 flex items-center justify-between text-xs text-muted">
          <span>
            {total} row{total === 1 ? '' : 's'} · tenant-scoped read
          </span>
          <span className="font-mono">
            limit {limit} · offset {offset}
            {schema.has_state_machine ? ' · state machine' : ''}
          </span>
        </footer>
      </div>
    </div>
  )
}
