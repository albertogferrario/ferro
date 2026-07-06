//! Conduit response DTOs — camelCase envelopes serialized to the wire.
//!
//! Never serialize SeaORM model types directly (RESEARCH Pitfall 3); these DTOs
//! own the camelCase renames and the nullable-field policy (Pitfall 4: keep
//! `Option<String>` → `null`, do NOT `skip_serializing_if`).

use serde::Serialize;

/// `{"user":{email,token,username,bio,image}}` inner object.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserDto {
    pub email: String,
    pub token: String,
    pub username: String,
    pub bio: Option<String>,
    pub image: Option<String>,
}

/// `{"profile":{username,bio,image,following}}` inner object.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileDto {
    pub username: String,
    pub bio: Option<String>,
    pub image: Option<String>,
    pub following: bool,
}

/// `{"article":{...}}` inner object. `tag_list`/`created_at`/`favorites_count`
/// serialize as `tagList`/`createdAt`/`favoritesCount`.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArticleDto {
    pub slug: String,
    pub title: String,
    pub description: String,
    pub body: String,
    pub tag_list: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
    pub favorited: bool,
    pub favorites_count: i64,
    pub author: ProfileDto,
}

/// `{"comment":{...}}` inner object.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommentDto {
    pub id: i64,
    pub created_at: String,
    pub updated_at: String,
    pub body: String,
    pub author: ProfileDto,
}
