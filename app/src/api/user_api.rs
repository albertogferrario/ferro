//! User API controller
//!
//! CRUD endpoints for the users resource under /api/v1/users.

use crate::models::users::{self, Entity as User};
use crate::resources::UserResource;
use ferro::serde::Deserialize;
use ferro::{handler, FormRequest, HttpResponse, Request, Response};
use sea_orm::{EntityTrait, PaginatorTrait};

/// Request body for creating a new user.
#[derive(Deserialize)]
pub struct CreateUserRequest {
    pub name: String,
    pub email: String,
    pub password: String,
}

impl ferro::Validate for CreateUserRequest {
    fn validate(&self) -> Result<(), ferro::validator::ValidationErrors> {
        Ok(())
    }
}

impl FormRequest for CreateUserRequest {}

/// Request body for updating an existing user (all fields optional).
#[derive(Deserialize)]
pub struct UpdateUserRequest {
    pub name: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
}

impl ferro::Validate for UpdateUserRequest {
    fn validate(&self) -> Result<(), ferro::validator::ValidationErrors> {
        Ok(())
    }
}

impl FormRequest for UpdateUserRequest {}

/// List users with pagination
///
/// GET /api/v1/users
#[handler]
pub async fn index(req: Request) -> Response {
    let page: u64 = req.query_as_or("page", 1u64).max(1);
    let per_page: u64 = req.query_as_or("per_page", 15u64).clamp(1, 100);
    let db = ferro::DB::connection().map_err(|e| {
        HttpResponse::json(ferro::serde_json::json!({"error": e.to_string()})).status(500)
    })?;
    let paginator = User::find().paginate(db.inner(), per_page);
    let total = paginator.num_items().await.map_err(|e| {
        HttpResponse::json(ferro::serde_json::json!({"error": e.to_string()})).status(500)
    })?;
    let items = paginator.fetch_page(page - 1).await.map_err(|e| {
        HttpResponse::json(ferro::serde_json::json!({"error": e.to_string()})).status(500)
    })?;
    let resources: Vec<UserResource> = items.into_iter().map(UserResource::from).collect();
    let meta = ferro::PaginationMeta::new(page, per_page, total);
    Ok(ferro::ResourceCollection::paginated(resources, meta).to_response(&req))
}

/// Show a single user
///
/// GET /api/v1/users/{id}
#[handler]
pub async fn show(req: Request, user: users::Model) -> Response {
    Ok(ferro::Resource::to_wrapped_response(
        &UserResource::from(user),
        &req,
    ))
}

/// Create a new user
///
/// POST /api/v1/users
#[handler]
pub async fn store(form: CreateUserRequest) -> Response {
    let model = users::Model::create()
        .set_name(form.name.clone())
        .set_email(form.email.clone())
        .set_password(form.password.clone())
        .insert()
        .await
        .map_err(|e| {
            HttpResponse::json(ferro::serde_json::json!({"error": e.to_string()})).status(500)
        })?;
    Ok(HttpResponse::json(ferro::serde_json::json!({"data": {"id": model.id}})).status(201))
}

/// Update an existing user
///
/// PUT /api/v1/users/{id}
#[handler]
pub async fn update(user: users::Model, form: UpdateUserRequest) -> Response {
    let mut builder = user.update();
    if let Some(ref v) = form.name {
        builder = builder.set_name(v.clone());
    }
    if let Some(ref v) = form.email {
        builder = builder.set_email(v.clone());
    }
    if let Some(ref v) = form.password {
        builder = builder.set_password(v.clone());
    }
    let updated = builder.save().await.map_err(|e| {
        HttpResponse::json(ferro::serde_json::json!({"error": e.to_string()})).status(500)
    })?;
    Ok(HttpResponse::json(
        ferro::serde_json::json!({"data": {"id": updated.id}}),
    ))
}

/// Delete a user
///
/// DELETE /api/v1/users/{id}
#[handler]
pub async fn destroy(user: users::Model) -> Response {
    user.delete().await.map_err(|e| {
        HttpResponse::json(ferro::serde_json::json!({"error": e.to_string()})).status(500)
    })?;
    Ok(HttpResponse::json(ferro::serde_json::json!({"message": "Deleted"})).status(200))
}
