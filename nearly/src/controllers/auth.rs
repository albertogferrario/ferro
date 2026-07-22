//! Password authentication: register, login, logout.

use ferro::serde_json::json;
use ferro::{handler, Auth, JsonUi, Redirect, Request, Response};
use sea_orm::Set;
use serde::Deserialize;

use ferro::database::ModelMut;

use crate::models::presence::{ActiveModel as PresenceActive, Entity as PresenceEntity};
use crate::models::profile::{ActiveModel as ProfileActive, Entity as ProfileEntity};
use crate::models::user::User;

/// Central Milan — where a brand-new user starts on the map.
const START_LAT: f64 = 45.4642;
const START_LNG: f64 = 9.19;

#[derive(Deserialize)]
struct LoginInput {
    email: String,
    password: String,
}

#[derive(Deserialize)]
struct RegisterInput {
    name: String,
    email: String,
    password: String,
}

/// GET /login
#[handler]
pub async fn login_page() -> Response {
    JsonUi::render_file("src/views/login.json", json!({}))
}

/// POST /login
#[handler]
pub async fn login(req: Request) -> Response {
    let input: LoginInput = req.form().await?;

    let invalid = || {
        JsonUi::render_file(
            "src/views/login.json",
            json!({ "email": "", "error": "Email o password non validi." }),
        )
        .map(|r| r.status(422))
    };

    let user = match User::find_by_email(&input.email).await? {
        Some(u) => u,
        None => return invalid(),
    };

    if !user.verify_password(&input.password)? {
        return invalid();
    }

    Auth::login(user.id as i64);
    Redirect::to("/map").into()
}

/// GET /register
#[handler]
pub async fn register_page() -> Response {
    JsonUi::render_file("src/views/register.json", json!({}))
}

/// POST /register — create the account plus its profile and starting presence.
#[handler]
pub async fn register(req: Request) -> Response {
    let input: RegisterInput = req.form().await?;

    let reject = |msg: &str| {
        JsonUi::render_file(
            "src/views/register.json",
            json!({ "name": "", "email": "", "error": msg }),
        )
        .map(|r| r.status(422))
    };

    if input.name.trim().len() < 2 {
        return reject("Inserisci un nome di almeno 2 caratteri.");
    }
    if !input.email.contains('@') {
        return reject("Inserisci un indirizzo email valido.");
    }
    if input.password.len() < 8 {
        return reject("La password deve avere almeno 8 caratteri.");
    }
    if User::find_by_email(&input.email).await?.is_some() {
        return reject("Questa email è già registrata.");
    }

    let user = User::create(input.name.trim(), input.email.trim(), &input.password).await?;
    let now = crate::models::now();

    let display_name = input
        .name
        .split_whitespace()
        .next()
        .unwrap_or("Utente")
        .to_string();
    let profile = ProfileActive {
        user_id: Set(user.id),
        display_name: Set(display_name),
        status: Set("Appena arrivato su Nearly 👋".to_string()),
        avatar_url: Set(None),
        visible: Set(true),
        created_at: Set(now.clone()),
        updated_at: Set(now.clone()),
        ..Default::default()
    };
    ProfileEntity::insert_one(profile).await.ok();

    let presence = PresenceActive {
        user_id: Set(user.id),
        lat: Set(START_LAT),
        lng: Set(START_LNG),
        last_seen: Set(now),
        ..Default::default()
    };
    PresenceEntity::insert_one(presence).await.ok();

    Auth::login(user.id as i64);
    Redirect::to("/map").into()
}

/// POST /logout
#[handler]
pub async fn logout() -> Response {
    Auth::logout();
    Redirect::to("/").into()
}
