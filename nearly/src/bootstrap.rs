//! Application bootstrap — global middleware and demo seed data.
//!
//! The UI is Inertia.js + React (see `frontend/`). The framework generates the
//! root HTML (with the Vite/manifest asset tags) and hands React the page
//! component + props; `ShareInertiaData` adds auth + CSRF to every response.

use ferro::{
    bind, global_middleware, ModelUserProvider, SessionConfig, SessionMiddleware, UserProvider, DB,
};

use crate::middleware;

/// Register global middleware and seed demo data.
pub async fn register() {
    // Database.
    DB::init().await.unwrap_or_else(|e| {
        eprintln!("Error: Failed to connect to database\n  Cause: {e}");
        eprintln!(
            "How to fix: set DATABASE_URL (e.g. sqlite://./nearly.db) and ensure it is writable."
        );
        std::process::exit(1);
    });

    // Fail fast in production if the frontend build is missing/mismatched, rather
    // than serving a blank page. No-op in development (the Vite dev server serves
    // assets), so `cargo run` with APP_ENV=local is unaffected.
    if let Err(e) = ferro::Inertia::preflight() {
        eprintln!("Error: Inertia frontend assets are not ready\n  Cause: {e}");
        eprintln!(
            "How to fix: run `cd frontend && npm install && npm run build`, or use \
             the Vite dev server (`npm run dev`) with APP_ENV=local."
        );
        std::process::exit(1);
    }

    // Auth: the generic provider hydrates `Auth::user()`/`Auth::user_as::<User>()`
    // by loading the user model by primary key — no hand-written provider needed
    // (User derives Authenticatable).
    bind!(
        dyn UserProvider,
        ModelUserProvider::<crate::models::user::Entity>::default()
    );

    // Global middleware. Session first so the cookie wraps everything downstream.
    global_middleware!(SessionMiddleware::new(SessionConfig::from_env()));
    global_middleware!(middleware::LoggingMiddleware);
    // Share auth user + CSRF token with every Inertia page.
    global_middleware!(middleware::ShareInertiaData);

    seed_demo_data().await;
}

/// Seed a living demo city (Milan) so the map is alive on first boot.
///
/// Runs only when the `users` table is empty. Demo login:
///   alex@nearly.app / password123
async fn seed_demo_data() {
    use crate::models::place::{ActiveModel as PlaceActive, Entity as PlaceEntity};
    use crate::models::presence::ActiveModel as PresenceActive;
    use crate::models::profile::ActiveModel as ProfileActive;
    use crate::models::trillo::{self, ActiveModel as TrilloActive};
    use crate::models::user::{Entity as UserEntity, User};
    use ferro::database::ModelMut;
    use sea_orm::{ActiveModelTrait, ActiveValue::Set, EntityTrait, PaginatorTrait};

    let db = match DB::connection() {
        Ok(c) => c,
        Err(_) => return,
    };

    let count = UserEntity::find().count(db.inner()).await.unwrap_or(1);
    if count > 0 {
        return;
    }

    let now = crate::models::now();

    // (name, email, display_name, status, lat, lng)
    let people = [
        (
            "Alex Rossi",
            "alex@nearly.app",
            "Alex",
            "Qui per esplorare la città 🧭",
            45.4641,
            9.1919,
        ),
        (
            "Giulia Bianchi",
            "giulia@nearly.app",
            "Giulia",
            "Nuova in città, cerco consigli e caffè ☕",
            45.4719,
            9.1880,
        ),
        (
            "Marco Verdi",
            "marco@nearly.app",
            "Marco",
            "Sempre pronto a due chiacchiere dal vivo",
            45.4520,
            9.1750,
        ),
        (
            "Sara Conti",
            "sara@nearly.app",
            "Sara",
            "Appassionata di jazz e mercatini",
            45.4869,
            9.1880,
        ),
        (
            "Luca Neri",
            "luca@nearly.app",
            "Luca",
            "Runner della domenica, qui per il brunch",
            45.4500,
            9.2050,
        ),
        (
            "Elena Ricci",
            "elena@nearly.app",
            "Elena",
            "Fotografa, catturo la città",
            45.4780,
            9.2270,
        ),
    ];

    let mut ids = Vec::new();
    for (name, email, display_name, status, lat, lng) in people {
        let user = match User::create(name, email, "password123").await {
            Ok(u) => u,
            Err(e) => {
                eprintln!("[seed] failed to create user {email}: {e}");
                return;
            }
        };
        ids.push(user.id);

        let profile = ProfileActive {
            user_id: Set(user.id),
            display_name: Set(display_name.to_string()),
            status: Set(status.to_string()),
            avatar_url: Set(None),
            visible: Set(true),
            created_at: Set(now.clone()),
            updated_at: Set(now.clone()),
            ..Default::default()
        };
        if let Err(e) = profile.insert(db.inner()).await {
            eprintln!("[seed] failed to insert profile for {email}: {e}");
        }

        let presence = PresenceActive {
            user_id: Set(user.id),
            lat: Set(lat),
            lng: Set(lng),
            last_seen: Set(now.clone()),
            ..Default::default()
        };
        if let Err(e) = presence.insert(db.inner()).await {
            eprintln!("[seed] failed to insert presence for {email}: {e}");
        }
    }

    // Places (trend + premium) around central Milan.
    let places = [
        ("Bar Luce", "Caffè", 45.4626, 9.1780, true),
        ("Fonderie Milanesi", "Bar", 45.4515, 9.1745, false),
        ("Biblioteca degli Alberi", "Parco", 45.4855, 9.1910, false),
        ("Mercato Centrale", "Food", 45.4870, 9.2040, true),
        ("Triennale", "Cultura", 45.4726, 9.1730, false),
    ];
    for (name, category, lat, lng, premium) in places {
        let place = PlaceActive {
            name: Set(name.to_string()),
            category: Set(category.to_string()),
            lat: Set(lat),
            lng: Set(lng),
            premium: Set(premium),
            created_at: Set(now.clone()),
            ..Default::default()
        };
        if let Err(e) = PlaceEntity::insert_one(place).await {
            eprintln!("[seed] failed to insert place {name}: {e}");
        }
    }

    // A pending trillo Giulia → Alex so the inbox is not empty on first login.
    if ids.len() >= 2 {
        let trillo = TrilloActive {
            from_user_id: Set(ids[1]),
            to_user_id: Set(ids[0]),
            status: Set(trillo::STATUS_PENDING.to_string()),
            created_at: Set(now.clone()),
            ..Default::default()
        };
        if let Err(e) = trillo.insert(db.inner()).await {
            eprintln!("[seed] failed to insert demo trillo: {e}");
        }
    }

    println!(
        "[seed] Nearly demo city seeded: {} people, {} places",
        ids.len(),
        places.len()
    );
}
