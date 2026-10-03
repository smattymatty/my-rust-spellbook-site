//! One Must Imagine — the simulation core.
//!
//! An idle game about pushers. The architecture is set by seven ADRs in
//! `game/docs/adr/`:
//!
//! - 0001 big-float numbers (`number`)
//! - 0002 per-universe state under a shared root (`state`)
//! - 0003 snapshot-out / commands-in boundary (`snapshot`, `command`)
//! - 0004 fixed-timestep accumulator (`sim`)
//! - 0005 capped offline progress (`sim`, `engine`)
//! - 0006 versioned save + migration chain (`save`)
//! - 0007 portable save + two-way version guard (`save`, `engine`)
//!
//! The core is pure Rust and fully tested natively. `wasm_api` is a thin
//! forwarder compiled only for the browser.

pub mod command;
pub mod content;
pub mod engine;
pub mod number;
pub mod save;
pub mod sim;
pub mod snapshot;
pub mod state;

#[cfg(target_arch = "wasm32")]
mod wasm_api;
