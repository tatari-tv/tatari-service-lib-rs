use std::time::{Duration, UNIX_EPOCH};

use super::*;
use crate::fake::env;

fn start() -> SystemTime {
    UNIX_EPOCH + Duration::from_secs(3600 * 24 * 365)
}

const START_RFC3339: &str = "1971-01-01T00:00:00Z";

#[test]
fn usable_returns_a_set_value() {
    assert_eq!(
        usable(&env(&[("GIT_BRANCH", "main")]), "GIT_BRANCH").as_deref(),
        Some("main")
    );
}

#[test]
fn usable_trims_whitespace() {
    assert_eq!(
        usable(&env(&[("GIT_BRANCH", "  main\n")]), "GIT_BRANCH").as_deref(),
        Some("main")
    );
}

#[test]
fn usable_treats_empty_as_unset() {
    assert_eq!(usable(&env(&[("GIT_BRANCH", "")]), "GIT_BRANCH"), None);
    assert_eq!(usable(&env(&[("GIT_BRANCH", "   ")]), "GIT_BRANCH"), None);
}

#[test]
fn usable_treats_unknown_as_unset() {
    assert_eq!(usable(&env(&[("GIT_BRANCH", "unknown")]), "GIT_BRANCH"), None);
}

#[test]
fn usable_treats_missing_as_unset() {
    assert_eq!(usable(&env(&[]), "GIT_BRANCH"), None);
}

#[test]
fn environment_lowercases_env() {
    assert_eq!(environment(&env(&[("ENV", "PROD"), ("CI", "true")])), "prod");
}

#[test]
fn environment_falls_back_to_ci_when_ci_is_set() {
    assert_eq!(environment(&env(&[("CI", "true")])), "ci");
}

#[test]
fn environment_falls_back_to_ci_when_env_is_unknown() {
    assert_eq!(environment(&env(&[("ENV", "unknown"), ("CI", "1")])), "ci");
}

#[test]
fn environment_falls_back_to_local_without_ci() {
    assert_eq!(environment(&env(&[])), "local");
}

#[test]
fn environment_treats_empty_ci_as_unset() {
    assert_eq!(environment(&env(&[("CI", "")])), "local");
}

#[test]
fn environment_treats_unknown_or_blank_ci_as_unset() {
    assert_eq!(environment(&env(&[("CI", "unknown")])), "local");
    assert_eq!(environment(&env(&[("CI", "  ")])), "local");
}

#[test]
fn deployed_at_keeps_fractional_seconds() {
    let lookup = env(&[("DEPLOY_DATETIME", "2026-10-30T23:46:31.250Z")]);
    assert_eq!(deployed_at(&lookup, start()), "2026-10-30T23:46:31.250Z");
}

#[test]
fn deployed_at_normalizes_rfc3339_to_utc() {
    let lookup = env(&[("DEPLOY_DATETIME", "2026-10-30T16:46:31-07:00")]);
    assert_eq!(deployed_at(&lookup, start()), "2026-10-30T23:46:31Z");
}

#[test]
fn deployed_at_keeps_a_utc_value() {
    let lookup = env(&[("DEPLOY_DATETIME", "2026-10-30T00:00:00Z")]);
    assert_eq!(deployed_at(&lookup, start()), "2026-10-30T00:00:00Z");
}

#[test]
fn deployed_at_parses_the_comma_fraction_form() {
    let lookup = env(&[("DEPLOY_DATETIME", "1970-01-01T01:00:00,000000000+00:00")]);
    assert_eq!(deployed_at(&lookup, start()), "1970-01-01T01:00:00Z");
}

#[test]
fn deployed_at_falls_back_to_start_when_unparseable() {
    let lookup = env(&[("DEPLOY_DATETIME", "yesterday")]);
    assert_eq!(deployed_at(&lookup, start()), START_RFC3339);
}

#[test]
fn deployed_at_falls_back_to_start_when_missing() {
    assert_eq!(deployed_at(&env(&[]), start()), START_RFC3339);
}

#[test]
fn deployed_at_falls_back_to_start_when_empty_or_unknown() {
    assert_eq!(deployed_at(&env(&[("DEPLOY_DATETIME", "")]), start()), START_RFC3339);
    assert_eq!(
        deployed_at(&env(&[("DEPLOY_DATETIME", "unknown")]), start()),
        START_RFC3339
    );
}

/// Captures every record whose message names `REPORT_TEST_KEY`.
struct Capture;

const REPORT_TEST_KEY: &str = "REPORT_TEST_KEY";

static CAPTURED: std::sync::Mutex<Vec<log::Level>> = std::sync::Mutex::new(Vec::new());

impl log::Log for Capture {
    fn enabled(&self, _: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        if record.args().to_string().contains(REPORT_TEST_KEY) {
            CAPTURED.lock().unwrap().push(record.level());
        }
    }

    fn flush(&self) {}
}

#[test]
fn report_unusable_warns_once_then_debugs() {
    let _ = log::set_logger(&Capture);
    log::set_max_level(log::LevelFilter::Trace);
    report_unusable(REPORT_TEST_KEY, format_args!("{REPORT_TEST_KEY} unusable"));
    report_unusable(REPORT_TEST_KEY, format_args!("{REPORT_TEST_KEY} unusable"));
    assert_eq!(*CAPTURED.lock().unwrap(), vec![log::Level::Warn, log::Level::Debug]);
}
