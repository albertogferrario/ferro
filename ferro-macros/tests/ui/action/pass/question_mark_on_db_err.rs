//! Compile-pass: `?` on `Result<_, sea_orm::DbErr>`.
//!
//! `From<sea_orm::DbErr> for ActionError` is unconditional in Plan 01
//! (sea-orm is an unconditional dep of framework). The fixture imports
//! `sea_orm::DbErr` directly via the `sea-orm` dev-dep declared in
//! `ferro-macros/Cargo.toml`.

#![allow(unused_imports)]

extern crate ferro_rs as ferro;

use ferro::{action, ActionResult, Request};

fn fallible() -> Result<i32, sea_orm::DbErr> {
    Ok(42)
}

#[action(redirect_to = "/dashboard")]
pub async fn h(_req: Request) -> ActionResult {
    let _n = fallible()?;
    Ok(())
}

fn main() {}
