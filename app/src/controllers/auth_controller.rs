use ferro::database::ModelMut;
use ferro::serde_json::json;
use ferro::{
    confirmed, email, handler, hash, json_response, min, required, session, session_mut, verify,
    Auth, HttpResponse, JsonUi, Request, Resource, Response, ResponseExt, Validator,
};
use sea_orm::Set;
use serde::Deserialize;

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

#[derive(Deserialize)]
#[allow(dead_code)]
struct LoginInput {
    email: String,
    password: String,
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
/// Login form rendered through JSON-UI (`src/views/login.json`) — the same
/// server-driven UI layer the rest of the app uses (see `pagamenti`), no
/// frontend build required. This is the page the OAuth `/authorize` endpoint
/// redirects unauthenticated browsers to (D-06 login reuse): a real MCP client
/// completing the browser-login flow lands here, submits the form, and is
/// redirected back into the OAuth flow via the `oauth_return_to` session value
/// the authorize handler stored.
#[handler]
pub async fn login_page(_req: Request) -> Response {
    JsonUi::render_file("src/views/login.json", json!({}))
}

/// POST /auth/login
///
/// Content-negotiated: a browser form submission (`application/x-www-form-urlencoded`)
/// establishes a session and 302-redirects (to `oauth_return_to` when present),
/// while a JSON body keeps the API contract (200/422 JSON for programmatic clients).
#[handler]
pub async fn login(req: Request) -> Response {
    let is_form = req
        .content_type()
        .map(|ct| ct.contains("form-urlencoded") || ct.contains("multipart/form-data"))
        .unwrap_or(false);

    if is_form {
        return login_form(req).await;
    }

    let input: LoginInput = req.json().await?;

    // Validate input
    let data = json!({
        "email": input.email,
        "password": input.password,
    });

    if let Err(errors) = Validator::new(&data)
        .rules("email", ferro::rules![required()])
        .rules("password", ferro::rules![required()])
        .validate()
    {
        return Err(HttpResponse::json(errors.to_json()).status(422));
    }

    match authenticate(&input.email, &input.password).await? {
        true => {
            // OAuth return-to: if login was initiated by /authorize, resume the OAuth flow.
            let return_to: Option<String> = session().and_then(|s| s.get("oauth_return_to"));
            if let Some(url) = return_to {
                session_mut(|s| {
                    s.forget("oauth_return_to");
                });
                return Ok(HttpResponse::new().status(302).header("Location", url));
            }

            // Fetch user data for response
            let user = User::find_by_email(&input.email)
                .await?
                .expect("User must exist after successful auth");

            json_response!({
                "user": {
                    "id": user.id,
                    "name": user.name,
                    "email": user.email
                }
            })
        }
        false => Err(HttpResponse::json(json!({
            "message": "The given data was invalid.",
            "errors": {
                "email": ["These credentials do not match our records."]
            }
        }))
        .status(422)),
    }
}

/// Browser-form login: authenticate, then 302 to `oauth_return_to` (or `/`).
/// On failure, re-render the form with an error and the submitted email
/// preserved (never the password).
async fn login_form(req: Request) -> Response {
    let input: LoginInput = req.form().await?;

    if authenticate(&input.email, &input.password).await? {
        let return_to: Option<String> = session().and_then(|s| s.get("oauth_return_to"));
        session_mut(|s| {
            s.forget("oauth_return_to");
        });
        let dest = return_to.unwrap_or_else(|| "/".to_string());
        return Ok(HttpResponse::new().status(302).header("Location", dest));
    }

    JsonUi::render_file(
        "src/views/login.json",
        json!({
            "email": input.email,
            "error": "These credentials do not match our records.",
        }),
    )
    .map(|resp| resp.status(422))
}

/// Verify an email/password pair, establishing the session on success.
async fn authenticate(email: &str, password: &str) -> Result<bool, HttpResponse> {
    let email = email.to_string();
    let password = password.to_string();
    let result = Auth::attempt(|| async {
        let user = match User::find_by_email(&email).await? {
            Some(u) => u,
            None => return Ok(None),
        };
        if verify(&password, &user.password)? {
            Ok(Some(user.id as i64))
        } else {
            Ok(None)
        }
    })
    .await?;
    Ok(result.is_some())
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

#[cfg(test)]
mod tests {
    use ferro::serde_json::Value;

    /// The login page is a JSON-UI view. Lock its core contract: valid JSON,
    /// the v2 schema, and a form that posts to `/auth/login` with the email +
    /// password fields the controller and OAuth flow depend on.
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
        assert_eq!(v["elements"]["password"]["props"]["field"], "password");
        assert_eq!(v["elements"]["password"]["props"]["input_type"], "password");
        // The email field pre-fills from handler data (preserved on a failed submit).
        assert_eq!(v["elements"]["email"]["props"]["data_path"], "/email");
    }
}
