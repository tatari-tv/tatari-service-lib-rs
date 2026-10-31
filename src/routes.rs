//! The three route bodies. Each public function reads the process env; the
//! `*_from` twin takes the lookup and clock so tests can pin both.

use std::time::SystemTime;

use log::debug;
use serde::Serialize;

use crate::env::{deployed_at, environment, report_unusable, usable};
use crate::{Build, start};

/// `GET /version`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Version {
    pub branch: String,
    pub revision: String,
    pub version: String,
}

/// `GET /deployed`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Deployed {
    pub deployed_at: String,
    /// Present only when `DEPLOYMENTS_LINK` is set: no pod sets it today, so
    /// the key would always be `""`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deployments_link: Option<String>,
    pub environment: String,
    /// `GIT_AUTHOR`; absent when the caller asked for privacy.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deployer: Option<String>,
}

/// `GET /status`.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Status {
    pub status: String,
    /// Seconds since [`init`](crate::init), rounded to two decimals.
    pub uptime: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

fn process_env(name: &str) -> Option<String> {
    std::env::var(name).ok()
}

/// The `/version` body: `GIT_BRANCH`, `GIT_VERSION`, `GIT_DESCRIBE`, each
/// falling back to `build` when unusable.
pub fn version(build: &Build) -> Version {
    version_from(&process_env, build)
}

/// The `/deployed` body; `privacy` drops `deployer`.
pub fn deployed(privacy: bool) -> Deployed {
    deployed_from(&process_env, privacy, start().system)
}

/// The `/status` body; `error` is emitted only when `Some` and non-empty.
pub fn status(status: &str, error: Option<String>) -> Status {
    status_from(status, error, start().instant.elapsed().as_secs_f64())
}

fn runtime_or_build(lookup: &impl Fn(&str) -> Option<String>, name: &'static str, fallback: &str) -> String {
    usable(lookup, name).unwrap_or_else(|| {
        report_unusable(
            name,
            format_args!("version: {name} unusable; using the build value {fallback:?}"),
        );
        fallback.to_string()
    })
}

pub(crate) fn version_from(lookup: &impl Fn(&str) -> Option<String>, build: &Build) -> Version {
    Version {
        branch: runtime_or_build(lookup, "GIT_BRANCH", build.branch),
        revision: runtime_or_build(lookup, "GIT_VERSION", build.revision),
        version: runtime_or_build(lookup, "GIT_DESCRIBE", build.describe),
    }
}

pub(crate) fn deployed_from(lookup: &impl Fn(&str) -> Option<String>, privacy: bool, start: SystemTime) -> Deployed {
    debug!("deployed: privacy={privacy}");
    Deployed {
        deployed_at: deployed_at(lookup, start),
        deployments_link: usable(lookup, "DEPLOYMENTS_LINK"),
        environment: environment(lookup),
        deployer: (!privacy).then(|| {
            usable(lookup, "GIT_AUTHOR").unwrap_or_else(|| {
                report_unusable(
                    "GIT_AUTHOR",
                    format_args!("deployed: GIT_AUTHOR unusable; deployer is empty"),
                );
                String::new()
            })
        }),
    }
}

pub(crate) fn status_from(status: &str, error: Option<String>, uptime_secs: f64) -> Status {
    Status {
        status: status.to_string(),
        uptime: (uptime_secs * 100.0).round() / 100.0,
        error: error.filter(|message| !message.is_empty()),
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests;
