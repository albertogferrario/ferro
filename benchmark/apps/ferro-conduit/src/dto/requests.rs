//! Conduit request DTOs — nested-envelope bodies (`{"user":{...}}` etc.).
//!
//! Every inbound body wraps the payload in a single-key envelope (Pitfall 5);
//! deserialize the envelope, then take `.user` / `.article` / `.comment`.
//! camelCase request fields (`tagList`) map to snake_case via `rename_all`.

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct RegisterEnvelope {
    pub user: RegisterReq,
}

#[derive(Debug, Deserialize)]
pub struct RegisterReq {
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub password: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LoginEnvelope {
    pub user: LoginReq,
}

#[derive(Debug, Deserialize)]
pub struct LoginReq {
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub password: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserEnvelope {
    pub user: UpdateUserReq,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserReq {
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub password: Option<String>,
    #[serde(default)]
    pub image: Option<String>,
    #[serde(default)]
    pub bio: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateArticleEnvelope {
    pub article: CreateArticleReq,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateArticleReq {
    pub title: String,
    pub description: String,
    pub body: String,
    #[serde(default)]
    pub tag_list: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateArticleEnvelope {
    pub article: UpdateArticleReq,
}

#[derive(Debug, Deserialize)]
pub struct UpdateArticleReq {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub body: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCommentEnvelope {
    pub comment: CreateCommentReq,
}

#[derive(Debug, Deserialize)]
pub struct CreateCommentReq {
    pub body: String,
}
