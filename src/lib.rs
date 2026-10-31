//! Tatari's standard service routes, `/version`, `/deployed` and `/status`, as
//! plain functions returning serde structs.
//!
//! A Rust port of `tatari_service_lib.routes` and `extractors` (the
//! `tatari-tv/python-tatari-service-lib` package). Every value is read at
//! runtime from the env the platform injects: the image env (`GIT_BRANCH`,
//! `GIT_VERSION`, `GIT_DESCRIBE`, `GIT_AUTHOR`) and the chart env (`ENV`,
//! `DEPLOY_DATETIME`, `DEPLOYMENTS_LINK`). The crate has no web framework
//! dependency: a service's own axum, actix or rocket handler wraps the struct
//! in its framework's JSON response.
//!
//! An env value that is empty or the literal `unknown` (the base image's
//! default for every `GIT_*`) counts as unset. `/version` then falls back to
//! the caller's compile-time [`Build`], and `deployed_at` to the process start.

#![deny(clippy::unwrap_used)]
#![deny(clippy::string_slice)]
#![deny(dead_code)]
#![deny(unused_variables)]

mod env;
#[cfg(test)]
mod fake;
mod routes;

use std::sync::OnceLock;
use std::time::{Instant, SystemTime};

use log::debug;

pub use routes::{Deployed, Status, Version, deployed, status, version};

/// The calling binary's compile-time git facts, from its own `build.rs`.
///
/// Used only when the runtime env does not carry a usable value, so a laptop
/// `cargo run` still answers `/version` with something true.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Build {
    pub branch: &'static str,
    pub revision: &'static str,
    pub describe: &'static str,
}

/// The process start, as a monotonic clock for `uptime` and a wall clock for
/// the `deployed_at` fallback.
#[derive(Debug, Clone, Copy)]
struct Start {
    instant: Instant,
    system: SystemTime,
}

static START: OnceLock<Start> = OnceLock::new();

/// The process start, captured on the first call and fixed after.
fn start() -> &'static Start {
    START.get_or_init(|| Start {
        instant: Instant::now(),
        system: SystemTime::now(),
    })
}

/// Capture the process start once; call it first thing in `main`.
///
/// Without it the start is captured by the first route call, so `uptime`
/// and the `deployed_at` fallback would measure from the first request.
pub fn init() {
    let start = start();
    debug!("init: process start captured at {:?}", start.system);
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests;
