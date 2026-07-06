//! Compile-fail: handler returns `Response` instead of `ActionResult`.
//! The natural Rust type error from the macro's wrapper
//! `let __action_result: ActionResult = async move { #fn_block }.await;`
//! should make this clear without a custom compile_error!.

#![allow(unused_imports)]

extern crate ferro_rs as ferro;

use ferro::{action, HttpResponse, Request, Response};

#[action(redirect_to = "/dashboard")]
pub async fn h(_req: Request) -> Response {
    Ok(HttpResponse::new())
}

fn main() {}
