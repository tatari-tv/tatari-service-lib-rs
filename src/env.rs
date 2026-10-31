//! The extractors: each standard-route value read from an injected env lookup,
//! so tests pass a map instead of mutating the process env.

use std::time::SystemTime;

use chrono::{DateTime, SecondsFormat, Utc};
use log::{debug, warn};

/// The literal the base image bakes into every `GIT_*` it was not given.
const UNSET_SENTINEL: &str = "unknown";

/// `name`'s value, or `None` when it is missing, empty or the base image's
/// `unknown` default.
pub(crate) fn usable(lookup: &impl Fn(&str) -> Option<String>, name: &str) -> Option<String> {
    lookup(name)
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty() && value != UNSET_SENTINEL)
}

/// `tatari_environments.get_environment()`: `ENV` lowercased, else `ci` when
/// `CI` is set, else `local`.
pub(crate) fn environment(lookup: &impl Fn(&str) -> Option<String>) -> String {
    match usable(lookup, "ENV") {
        Some(env) => env.to_lowercase(),
        None if usable(lookup, "CI").is_some() => "ci".to_string(),
        None => "local".to_string(),
    }
}

/// `DEPLOY_DATETIME` as RFC 3339 UTC at its own precision, else `start`.
///
/// Accepts RFC 3339 and the comma-fraction form
/// (`1970-01-01T01:00:00,000000000+00:00`) Python's `fromisoformat` takes. A
/// value that parses as neither logs a warning and falls back, as the Python
/// lib does.
pub(crate) fn deployed_at(lookup: &impl Fn(&str) -> Option<String>, start: SystemTime) -> String {
    let parsed = match usable(lookup, "DEPLOY_DATETIME") {
        None => {
            debug!("deployed_at: DEPLOY_DATETIME unset; using the process start");
            None
        }
        Some(raw) => match DateTime::parse_from_rfc3339(&raw.replacen(',', ".", 1)) {
            Ok(dt) => Some(dt.with_timezone(&Utc)),
            Err(err) => {
                warn!("unable to parse DEPLOY_DATETIME={raw:?} ({err}); using the process start");
                None
            }
        },
    };
    parsed
        .unwrap_or_else(|| DateTime::<Utc>::from(start))
        .to_rfc3339_opts(SecondsFormat::AutoSi, true)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests;
