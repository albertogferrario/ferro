//! Typed offload handle and serializable-contract enforcement.
//!
//! This module provides the four primitives for `#[offload]`-annotated service
//! methods:
//!
//! - [`OffloadSerializable`] — compile-time marker bounding every type that
//!   crosses the offload isolation boundary.
//! - [`HandleKey`] — opaque UUID v4-backed identity minted at enqueue.
//! - [`OffloadHandle<T>`] — inert typed handle carrying the enqueue identity and
//!   the success type (phantom, zero-cost). Resolve and subscribe surface arrives
//!   in Phases 246 and 247.
//! - [`Offloadable`] — the enqueue trait the macro emits per offloaded method;
//!   the `offload()` body is a provided default so per-method emission stays
//!   minimal.

use crate::Error;
use async_trait::async_trait;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::marker::PhantomData;
use uuid::Uuid;

/// Marker trait: every parameter and return type crossing the `#[offload]`
/// isolation boundary must be `Serialize + DeserializeOwned`. A blanket impl
/// covers all qualifying types; the `#[diagnostic::on_unimplemented]` attribute
/// rewrites the compiler error to name the offending type and frame the isolation
/// boundary.
#[diagnostic::on_unimplemented(
    message = "`{Self}` crosses the #[offload] isolation boundary and must be `Serialize + DeserializeOwned`",
    note = "offloaded parameters and return types travel as a queue payload; implement `Serialize` and `DeserializeOwned` for `{Self}` to seal the module across the isolation boundary"
)]
pub trait OffloadSerializable: Serialize + DeserializeOwned {}

impl<T: Serialize + DeserializeOwned> OffloadSerializable for T {}

/// Opaque identity of a single offload enqueue, minted as a fresh UUID v4.
///
/// Non-secret: uniqueness (not unpredictability) is the only requirement in
/// Phase 245. Decoupled from [`Job::idempotency_key()`](crate::Job::idempotency_key)
/// — the handle identifies *this* offload call, not the result content (D-07).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HandleKey(String);

impl HandleKey {
    /// Mint a fresh handle key from a UUID v4.
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    /// The key as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for HandleKey {
    fn default() -> Self {
        Self::new()
    }
}

/// A typed handle to an offloaded call's eventual result.
///
/// Inert in Phase 245: it carries the enqueue identity ([`HandleKey`]) and the
/// success type `T` (phantom, zero-cost), but exposes no resolve or subscribe
/// surface — those arrive with the result path (Phase 246) and streaming (Phase 247).
///
/// `PhantomData<fn() -> T>` is used instead of `PhantomData<T>` so the handle is
/// `Send + Sync` regardless of `T`. The phantom is `#[serde(skip)]` so serde does
/// not require `T: Serialize`, allowing the handle to round-trip even when `T` is
/// not serializable (OFFLOAD-02e).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OffloadHandle<T> {
    key: HandleKey,
    #[serde(skip)]
    _phantom: PhantomData<fn() -> T>,
}

impl<T> OffloadHandle<T> {
    /// Wrap a minted key in a typed handle.
    pub fn new(key: HandleKey) -> Self {
        Self {
            key,
            _phantom: PhantomData,
        }
    }

    /// The handle key as a string slice (the eventual projection/subscription key).
    pub fn key(&self) -> &str {
        self.key.as_str()
    }

    /// The handle key newtype.
    pub fn id(&self) -> &HandleKey {
        &self.key
    }
}

/// Enqueue entrypoint layered on the derived Job.
///
/// The `#[offload]` trait method itself stays `-> T` in-process (D-01);
/// `.offload()` is the enqueue path that returns a typed handle to where the
/// result will eventually land.
///
/// The macro emits `impl Offloadable for <..>Job { type Output = <T>; }`;
/// the `offload()` body is a provided default so per-method emission stays minimal.
/// Bounds are on the trait itself (`Serialize + DeserializeOwned + Sized`) so a bare
/// `impl Offloadable for XJob { type Output = ...; }` carries no extra where-clause.
#[async_trait]
pub trait Offloadable: crate::Job + Serialize + DeserializeOwned + Sized {
    /// The method's success type (D-09): `T` for `-> T` or `Result<T, E>`,
    /// `()` for `-> ()` or no return. Enforced to implement [`OffloadSerializable`].
    type Output: OffloadSerializable;

    /// Enqueue this job and return a typed handle.
    ///
    /// Enqueue can fail (DB insert), mirroring `dispatch` (D-02). The handle's
    /// key is minted fresh at each call, independent of `idempotency_key()` (D-07).
    async fn offload(self) -> Result<OffloadHandle<Self::Output>, Error> {
        let key = HandleKey::new();
        crate::PendingDispatch::new(self).dispatch().await?;
        Ok(OffloadHandle::new(key))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // OFFLOAD-02d: the key accessor returns a valid UUID v4 string.
    #[test]
    fn handle_key_is_uuid_v4() {
        let k = HandleKey::new();
        let parsed = Uuid::parse_str(k.as_str()).expect("HandleKey must be a valid UUID");
        assert_eq!(parsed.get_version(), Some(uuid::Version::Random));
    }

    // A struct that is NOT Serialize/Deserialize, used to prove the handle serializes anyway.
    struct NotSerializable;

    // OFFLOAD-02e: OffloadHandle<T> serde round-trips even when T: !Serialize.
    #[test]
    fn handle_round_trips_with_non_serializable_t() {
        let key = HandleKey::new();
        let handle: OffloadHandle<NotSerializable> = OffloadHandle::new(key.clone());
        let json = serde_json::to_string(&handle).expect("handle must serialize");
        let back: OffloadHandle<NotSerializable> =
            serde_json::from_str(&json).expect("handle must deserialize");
        assert_eq!(back.id(), &key);
        assert_eq!(back.key(), key.as_str());
    }
}
