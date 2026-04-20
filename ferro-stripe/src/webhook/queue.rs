//! Queue-based Stripe webhook dispatch (eventual-consistency path).
//!
//! Phase 141 relocates `ProcessStripeWebhook` here and wires it to
//! `SyncDispatcher`. In Phase 140 the file exists as an empty module
//! so the directory structure matches the design doc §3.1.

// Phase 141: ProcessStripeWebhook job relocated here.
