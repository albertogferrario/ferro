//! Dashboard controller

use ferro::{Auth, Inertia, InertiaProps, Model, Request, Response};
use serde::Serialize;

use crate::models::user::Entity as UserEntity;

#[derive(Serialize)]
pub struct UserInfo {
    pub id: i64,
    pub name: String,
    pub email: String,
}

#[derive(InertiaProps)]
pub struct DashboardProps {
    pub user: UserInfo,
}

pub async fn index(req: Request) -> Response {
    // Get the authenticated user
    let user_id = Auth::id().expect("User must be authenticated");

    let user = UserEntity::find_by_pk(user_id)
        .await?
        .expect("User must exist");

    Inertia::render(
        &req,
        "Dashboard",
        DashboardProps {
            user: UserInfo {
                id: user.id,
                name: user.name,
                email: user.email,
            },
        },
    )
}
