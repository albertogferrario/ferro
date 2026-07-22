import { useEffect, useRef } from 'react'

type Handler = (event: string, data: any) => void

/**
 * Subscribe to a Ferro broadcast channel over the framework's `/_ferro/ws`
 * endpoint and invoke `onEvent(event, data)` for each server event.
 *
 * - Public channels subscribe directly.
 * - `private-` / `presence-` channels first POST to `/broadcasting/auth`
 *   (session cookie + CSRF header) to obtain a signed token, then subscribe.
 *
 * Pass `channel = null` to disable (e.g. before the user is known).
 */
export function useChannel(channel: string | null, onEvent: Handler) {
  const handler = useRef(onEvent)
  handler.current = onEvent

  useEffect(() => {
    if (!channel) return
    const proto = location.protocol === 'https:' ? 'wss' : 'ws'
    const ws = new WebSocket(`${proto}://${location.host}/_ferro/ws`)
    const csrf = document.querySelector('meta[name="csrf-token"]')?.getAttribute('content') ?? ''
    let closed = false

    ws.onmessage = async (ev) => {
      let msg: any
      try {
        msg = JSON.parse(ev.data)
      } catch {
        return
      }
      if (msg.type === 'connected') {
        if (channel.startsWith('private-') || channel.startsWith('presence-')) {
          try {
            const res = await fetch('/broadcasting/auth', {
              method: 'POST',
              credentials: 'include',
              headers: { 'Content-Type': 'application/json', 'X-CSRF-TOKEN': csrf },
              body: JSON.stringify({ channel_name: channel, socket_id: msg.socket_id }),
            })
            if (!res.ok || closed) return
            const auth = await res.json()
            ws.send(
              JSON.stringify({ type: 'subscribe', channel, auth: auth.auth, channel_data: auth.channel_data }),
            )
          } catch {
            /* auth failed — stay unsubscribed */
          }
        } else {
          ws.send(JSON.stringify({ type: 'subscribe', channel }))
        }
      } else if (msg.type === 'event') {
        handler.current(msg.event, msg.data)
      }
    }

    return () => {
      closed = true
      ws.close()
    }
  }, [channel])
}
