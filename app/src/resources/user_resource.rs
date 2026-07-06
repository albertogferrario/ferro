use ferro::ApiResource;

#[derive(ApiResource)]
#[resource(model = "crate::models::entities::users::Model")]
#[allow(dead_code)]
pub struct UserResource {
    pub id: i32,
    pub name: String,
    pub email: String,
    #[resource(skip)]
    pub password: String,
    #[resource(skip)]
    pub remember_token: Option<String>,
    #[resource(skip)]
    pub created_at: String,
    #[resource(skip)]
    pub updated_at: String,
}
