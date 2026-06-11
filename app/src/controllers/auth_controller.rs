use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use ferro::config::env::Environment;
use ferro::database::ModelMut;
use ferro::serde_json::json;
use ferro::{
    confirmed, email, handler, hash, json_response, min, required, AppConfig, Auth, Cache,
    HttpResponse, JsonUi, Request, Resource, Response, ResponseExt, Validator,
};
use ferro_mcp_oauth::oauth_resume_redirect;
use rand::Rng;
use sea_orm::Set;
use serde::Deserialize;
use std::time::Duration;

use crate::models::users;
use crate::models::users::User;
use crate::resources::UserResource;

#[derive(Deserialize)]
#[allow(dead_code)]
struct RegisterInput {
    name: String,
    email: String,
    password: String,
    password_confirmation: String,
}

/// Input for the magic-link request form: email only, no password.
#[derive(Deserialize)]
struct RequestLinkInput {
    email: String,
}

/// POST /auth/register
///
/// Validates input, hashes password, creates user, and logs in.
#[handler]
pub async fn register(req: Request) -> Response {
    let input: RegisterInput = req.json().await?;

    // Validate input
    let data = json!({
        "name": input.name,
        "email": input.email,
        "password": input.password,
        "password_confirmation": input.password_confirmation,
    });

    if let Err(errors) = Validator::new(&data)
        .rules("name", ferro::rules![required()])
        .rules("email", ferro::rules![required(), email()])
        .rules("password", ferro::rules![required(), min(8), confirmed()])
        .validate()
    {
        return Err(HttpResponse::json(errors.to_json()).status(422));
    }

    // Check email uniqueness
    if User::find_by_email(&input.email).await?.is_some() {
        return Err(HttpResponse::json(json!({
            "message": "The given data was invalid.",
            "errors": {
                "email": ["The email has already been taken."]
            }
        }))
        .status(422));
    }

    // Hash password
    let hashed = hash(&input.password)?;

    // Insert new user
    let new_user = users::ActiveModel {
        name: Set(input.name),
        email: Set(input.email),
        password: Set(hashed),
        ..Default::default()
    };

    let user = users::Entity::insert_one(new_user).await?;

    // Log in
    Auth::login(user.id as i64);

    // Return 201 with user data (no password)
    json_response!({
        "user": {
            "id": user.id,
            "name": user.name,
            "email": user.email
        }
    })
    .status(201)
}

/// GET /auth/login
///
/// Login form rendered through JSON-UI (`src/views/login.json`) — email-only
/// "send login link" form. This is the page the OAuth `/authorize` endpoint
/// redirects unauthenticated browsers to (D-06 login reuse): a real MCP client
/// completing the browser-login flow lands here, submits their email, and receives
/// a magic-link that resumes the OAuth flow via the `oauth_return_to` session value
/// the authorize handler stored.
#[handler]
pub async fn login_page(_req: Request) -> Response {
    JsonUi::render_file("src/views/login.json", json!({}))
}

/// POST /auth/login — magic-link request handler.
///
/// Accepts an email address, looks up the user, generates a single-use
/// TTL-bounded token stored in `ferro-cache`, and either surfaces the verify
/// link directly (dev mode) or dispatches it via mail (non-dev, best-effort).
/// Renders `login_confirm.json` on success; re-renders `login.json` with an
/// error when the email is not registered (T-202-04 accepted flag — reveals
/// account existence; acceptable for the sample exemplar).
#[handler]
pub async fn login(req: Request) -> Response {
    let input: RequestLinkInput = req.form().await?;

    // Look up the user by email.
    let user = match User::find_by_email(&input.email).await? {
        Some(u) => u,
        None => {
            return JsonUi::render_file(
                "src/views/login.json",
                json!({
                    "email": input.email,
                    "error": "No account found for this email.",
                }),
            )
            .map(|resp| resp.status(422));
        }
    };

    let user_id: i64 = user.id as i64;
    let token = generate_magic_link_token();
    let key = format!("magic_link:{token}");

    // Store token in cache: single-use, 15-minute TTL.
    Cache::put(&key, &user_id, Some(Duration::from_secs(15 * 60))).await?;

    let base_url = AppConfig::from_env().url;
    let verify_url = format!("{base_url}/auth/verify?token={token}");

    let env = Environment::detect();
    if env.is_development() {
        tracing::info!(magic_link = %verify_url, "Magic-link generated (dev mode)");
        JsonUi::render_file(
            "src/views/login_confirm.json",
            json!({
                "dev_mode": true,
                "dev_link": verify_url,
                "dev_link_label": "Open login link (dev only)"
            }),
        )
    } else {
        // Non-dev: best-effort mail dispatch — never a hard failure.
        // SMTP is not required for CI or app boot; errors are logged as warnings.
        // The mail path is documented but not exercised by tests (T-202-MAIL).
        send_magic_link_mail_best_effort(&user.email, &verify_url).await;
        JsonUi::render_file(
            "src/views/login_confirm.json",
            json!({
                "dev_mode": false,
                "dev_link": "",
                "dev_link_label": ""
            }),
        )
    }
}

/// GET /auth/verify?token=... — magic-link verification.
///
/// Single-use: the token is consumed (forget) BEFORE the session is established,
/// so a clicked link cannot be replayed (T-202-01). An absent or expired token
/// re-renders the request-link page with an error (T-202-02).
#[handler]
pub async fn verify_magic_link(req: Request) -> Response {
    let token = match req.query("token") {
        Some(t) => t,
        None => {
            return JsonUi::render_file(
                "src/views/login.json",
                json!({ "error": "This login link has expired or is invalid. Request a new one." }),
            )
            .map(|resp| resp.status(422));
        }
    };

    let key = format!("magic_link:{token}");

    // Single-use: get then forget BEFORE validation (mirrors token.rs lines 62-64).
    let user_id: Option<i64> = Cache::get(&key).await.ok().flatten();
    let _ = Cache::forget(&key).await;

    let user_id = match user_id {
        Some(id) => id,
        None => {
            return JsonUi::render_file(
                "src/views/login.json",
                json!({ "error": "This login link has expired or is invalid. Request a new one." }),
            )
            .map(|resp| resp.status(422));
        }
    };

    Auth::login(user_id);
    return oauth_resume_redirect("/");
}

/// POST /auth/logout
///
/// Clears the authenticated session.
#[handler]
pub async fn logout(_req: Request) -> Response {
    Auth::logout();

    json_response!({
        "message": "Logged out successfully."
    })
}

/// GET /auth/profile
///
/// Returns the authenticated user's profile as a UserResource.
/// Uses AuthUser extractor (auto 401) and API Resource for response shaping.
#[handler]
pub async fn profile(req: Request) -> Response {
    let user = Auth::user_as::<users::Model>()
        .await?
        .ok_or_else(|| HttpResponse::json(json!({"message": "Unauthenticated."})).status(401))?;
    let resource = UserResource::from(user);
    Ok(resource.to_wrapped_response(&req))
}

/// Generate a high-entropy URL-safe magic-link token.
///
/// Replicates `ferro-mcp-oauth::pkce::generate_auth_code` locally: 256 bits of
/// random entropy encoded as 43 URL-safe base64 characters (T-202-03).
fn generate_magic_link_token() -> String {
    let bytes: [u8; 32] = rand::thread_rng().gen();
    URL_SAFE_NO_PAD.encode(bytes)
}

/// Dispatch the magic-link verify URL via mail (non-dev only).
///
/// Best-effort: any error is logged as a warning. SMTP is not required
/// for CI or app boot — this branch is never reached when `APP_ENV=local`.
async fn send_magic_link_mail_best_effort(to_email: &str, verify_url: &str) {
    use ferro::{
        MailMessage, Notifiable, Notification, NotificationChannel, NotificationDispatcher,
    };

    struct MagicLinkNotification {
        verify_url: String,
    }

    impl Notification for MagicLinkNotification {
        fn via(&self) -> Vec<NotificationChannel> {
            vec![NotificationChannel::Mail]
        }
        fn to_mail(&self) -> Option<MailMessage> {
            Some(
                MailMessage::new()
                    .subject("Your login link".to_string())
                    .body(format!(
                        "Click the link below to sign in. It expires in 15 minutes.\n\n{}",
                        self.verify_url
                    )),
            )
        }
        fn notification_type(&self) -> &'static str {
            "MagicLink"
        }
    }

    struct MailRecipient {
        email: String,
    }

    impl Notifiable for MailRecipient {
        fn route_notification_for(&self, channel: NotificationChannel) -> Option<String> {
            match channel {
                NotificationChannel::Mail => Some(self.email.clone()),
                _ => None,
            }
        }
        fn notifiable_id(&self) -> String {
            self.email.clone()
        }
        fn notifiable_type(&self) -> &'static str {
            "User"
        }
    }

    let recipient = MailRecipient {
        email: to_email.to_string(),
    };
    let notification = MagicLinkNotification {
        verify_url: verify_url.to_string(),
    };

    if let Err(e) = NotificationDispatcher::send(&recipient, notification).await {
        tracing::warn!(error = %e, "Magic-link mail dispatch failed (non-dev); continuing.");
    }
}

#[cfg(test)]
mod tests {
    use ferro::serde_json::Value;

    /// The login page is a JSON-UI view. Lock its core contract: valid JSON,
    /// the v2 schema, and an email-only form that posts to `/auth/login`.
    /// The password field was removed when login was converted to magic-link.
    #[test]
    fn login_view_is_valid_and_posts_to_login() {
        let raw = include_str!("../views/login.json");
        let v: Value = ferro::serde_json::from_str(raw).expect("login.json must be valid JSON");

        assert_eq!(v["$schema"], "ferro-json-ui/v2");
        assert_eq!(
            v["elements"]["form"]["props"]["action"]["handler"],
            "/auth/login"
        );
        assert_eq!(v["elements"]["email"]["props"]["field"], "email");
        assert_eq!(v["elements"]["email"]["props"]["input_type"], "email");
        // Password field must not exist — login is magic-link only.
        assert!(
            v["elements"]["password"].is_null(),
            "login.json must not contain a password field"
        );
        // The email field pre-fills from handler data (preserved on a failed submit).
        assert_eq!(v["elements"]["email"]["props"]["data_path"], "/email");
        // Submit button label must match the magic-link CTA.
        assert_eq!(v["elements"]["submit"]["props"]["label"], "Send login link");
    }
}
