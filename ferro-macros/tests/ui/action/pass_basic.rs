//! Compile-pass: minimal `#[action]` handler with bare `?` over the four
//! canonical error sources.

extern crate ferro_rs as ferro;

use ferro::{action, ActionError, ActionResult, FrameworkError, Request};

fn fallible_framework() -> Result<(), FrameworkError> {
    Ok(())
}

fn fallible_string() -> Result<(), String> {
    Ok(())
}

fn fallible_str() -> Result<(), &'static str> {
    Ok(())
}

async fn fallible_db() -> Result<(), sea_orm::DbErr> {
    Ok(())
}

#[action(redirect_to = "/dashboard")]
pub async fn happy(_req: Request) -> ActionResult {
    fallible_framework()?;
    fallible_string()?;
    fallible_str()?;
    fallible_db().await?;

    if false {
        return Err(ActionError::not_found("nope"));
    }
    Ok(())
}

fn main() {}
