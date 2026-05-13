//! `GuardedUpdate<E>` — chainable builder for atomic conditional `UPDATE`
//! statements. Body lands in plan 152-03; this file is a stub for plan 152-01
//! so the crate compiles and downstream plans can register the crate before
//! the builder body exists.

#![allow(dead_code)]

use std::marker::PhantomData;

use sea_orm::EntityTrait;

pub struct GuardedUpdate<E: EntityTrait> {
    _entity: PhantomData<E>,
}
