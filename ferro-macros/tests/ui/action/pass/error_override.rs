//! Compile-pass: returns Err with redirect override (D-08 — consumer supplies the auth path).

#![allow(unused_imports)]

extern crate ferro_rs as ferro;

use ferro::{action, ActionError, ActionResult, Request};

#[action(redirect_to = "/dashboard")]
pub async fn h(_req: Request) -> ActionResult {
    Err(ActionError::unauthorized("login required").redirect_to("/your-login-path"))
}

fn main() {}
