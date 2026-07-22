import { Head, Link, router, usePage } from '@inertiajs/react'
import { MapContainer, Marker, Popup, TileLayer } from 'react-leaflet'
import L from 'leaflet'
import Layout from '../Layout'

type Person = { user_id: number; name: string; status: string; lat: number; lng: number }
type Place = { id: number; name: string; category: string; premium: boolean; lat: number; lng: number }

function pin(kind: 'person' | 'place' | 'premium') {
  return L.divIcon({
    className: '',
    html: `<div class="pin pin--${kind}"></div>`,
    iconSize: [26, 26],
    iconAnchor: [13, 26],
    popupAnchor: [0, -26],
  })
}

export default function Map() {
  const { center, people, places } = usePage().props as unknown as {
    center: [number, number]
    people: Person[]
    places: Place[]
  }

  return (
    <Layout active="map" bare>
      <Head title="Mappa — Nearly" />
      <div className="map-wrap">
        <div className="map-count">
          {people.length} persone · {places.length} luoghi
        </div>

        <MapContainer center={center} zoom={14} zoomControl={false} style={{ height: '100%', width: '100%' }}>
          <TileLayer
            url="https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png"
            attribution="&copy; OpenStreetMap"
          />

          {people.map((p) => (
            <Marker key={`u${p.user_id}`} position={[p.lat, p.lng]} icon={pin('person')}>
              <Popup>
                <div className="popup-name">{p.name}</div>
                <div className="popup-status">{p.status}</div>
                <Link href={`/utenti/${p.user_id}`} className="btn btn--primary" style={{ padding: '8px 12px' }}>
                  Vedi profilo →
                </Link>
              </Popup>
            </Marker>
          ))}

          {places.map((pl) => (
            <Marker key={`p${pl.id}`} position={[pl.lat, pl.lng]} icon={pin(pl.premium ? 'premium' : 'place')}>
              <Popup>
                <div className="popup-name">{pl.name}</div>
                <div className="popup-status">
                  {pl.category}
                  {pl.premium ? ' · ⭐ Premium' : ''}
                </div>
              </Popup>
            </Marker>
          ))}
        </MapContainer>

        <button className="btn btn--primary map-fab" onClick={() => router.post('/presence/checkin')}>
          Sono ancora qui 📍
        </button>
      </div>
    </Layout>
  )
}
