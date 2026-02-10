use ferro::database::ModelMut;
use ferro::serde_json::json;
use ferro::{
    confirmed, email, handler, hash, json_response, min, required, verify, Auth, HttpResponse,
    Request, Resource, Response, ResponseExt, Validator,
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

/// POST /auth/login
///
/// Validates credentials and authenticates user via Auth::attempt.
#[handler]
pub async fn login(req: Request) -> Response {
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

    let email = input.email.clone();
    let password = input.password.clone();

    // Attempt authentication
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

    match result {
        Some(_) => {
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
        None => Err(HttpResponse::json(json!({
            "message": "The given data was invalid.",
            "errors": {
                "email": ["These credentials do not match our records."]
            }
        }))
        .status(422)),
    }
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
