use std::time::{Duration, UNIX_EPOCH};

use serde_json::json;

use super::*;
use crate::fake::env;

const BUILD: Build = Build {
    branch: "build-branch",
    revision: "build-revision",
    describe: "v0.0.0-build",
};

fn start() -> SystemTime {
    UNIX_EPOCH + Duration::from_secs(3600 * 24 * 365)
}

const FULL_ENV: &[(&str, &str)] = &[
    ("GIT_BRANCH", "main"),
    ("GIT_VERSION", "fa87ee37083df524ad2ca39826893ce1af482e78"),
    ("GIT_DESCRIBE", "v0.1.1-5-gfa87ee3"),
    ("GIT_AUTHOR", "escote-tatari"),
    ("DEPLOY_DATETIME", "2026-10-30T23:46:31Z"),
    ("DEPLOYMENTS_LINK", "https://example.test/deployments"),
    ("ENV", "Prod"),
];

#[test]
fn version_reads_the_runtime_git_vars() {
    let got = version_from(&env(FULL_ENV), &BUILD);
    assert_eq!(
        got,
        Version {
            branch: "main".to_string(),
            revision: "fa87ee37083df524ad2ca39826893ce1af482e78".to_string(),
            version: "v0.1.1-5-gfa87ee3".to_string(),
        }
    );
}

#[test]
fn version_falls_back_to_the_build_when_runtime_is_unknown() {
    let lookup = env(&[
        ("GIT_BRANCH", "unknown"),
        ("GIT_VERSION", "unknown"),
        ("GIT_DESCRIBE", "unknown"),
    ]);
    let got = version_from(&lookup, &BUILD);
    assert_eq!(got.branch, BUILD.branch);
    assert_eq!(got.revision, BUILD.revision);
    assert_eq!(got.version, BUILD.describe);
}

#[test]
fn version_falls_back_to_the_build_when_runtime_is_empty() {
    let lookup = env(&[("GIT_BRANCH", ""), ("GIT_VERSION", ""), ("GIT_DESCRIBE", "")]);
    let got = version_from(&lookup, &BUILD);
    assert_eq!(got.branch, BUILD.branch);
    assert_eq!(got.revision, BUILD.revision);
    assert_eq!(got.version, BUILD.describe);
}

#[test]
fn version_falls_back_to_the_build_when_runtime_is_missing() {
    let got = version_from(&env(&[]), &BUILD);
    assert_eq!(got.branch, BUILD.branch);
    assert_eq!(got.revision, BUILD.revision);
    assert_eq!(got.version, BUILD.describe);
}

#[test]
fn version_json_shape() {
    let value = serde_json::to_value(version_from(&env(FULL_ENV), &BUILD)).unwrap();
    assert_eq!(
        value,
        json!({
            "branch": "main",
            "revision": "fa87ee37083df524ad2ca39826893ce1af482e78",
            "version": "v0.1.1-5-gfa87ee3",
        })
    );
}

#[test]
fn deployed_json_shape_with_every_key() {
    let got = deployed_from(&env(FULL_ENV), false, start());
    assert_eq!(
        serde_json::to_string(&got).unwrap(),
        r#"{"deployed_at":"2026-10-30T23:46:31Z","deployments_link":"https://example.test/deployments","environment":"prod","deployer":"escote-tatari"}"#
    );
    let value = serde_json::to_value(got).unwrap();
    assert_eq!(
        value,
        json!({
            "deployed_at": "2026-10-30T23:46:31Z",
            "deployments_link": "https://example.test/deployments",
            "environment": "prod",
            "deployer": "escote-tatari",
        })
    );
}

#[test]
fn deployed_privacy_omits_the_deployer() {
    let got = deployed_from(&env(FULL_ENV), true, start());
    assert_eq!(got.deployer, None);
    let value = serde_json::to_value(got).unwrap();
    assert!(value.get("deployer").is_none());
}

#[test]
fn deployed_without_privacy_keeps_an_empty_deployer() {
    let got = deployed_from(&env(&[("GIT_AUTHOR", "unknown")]), false, start());
    assert_eq!(got.deployer.as_deref(), Some(""));
}

#[test]
fn deployed_omits_deployments_link_when_unset() {
    let got = deployed_from(&env(&[]), false, start());
    assert_eq!(got.deployments_link, None);
    assert_eq!(
        serde_json::to_string(&got).unwrap(),
        r#"{"deployed_at":"1971-01-01T00:00:00Z","environment":"local","deployer":""}"#
    );
}

#[test]
fn deployed_omits_deployments_link_when_unknown() {
    let got = deployed_from(&env(&[("DEPLOYMENTS_LINK", "unknown")]), false, start());
    assert_eq!(got.deployments_link, None);
}

#[test]
fn deployed_falls_back_to_start_and_local() {
    let got = deployed_from(&env(&[]), false, start());
    assert_eq!(got.deployed_at, "1971-01-01T00:00:00Z");
    assert_eq!(got.environment, "local");
}

#[test]
fn status_json_shape_without_error() {
    let value = serde_json::to_value(status_from("ok", None, 12.345_678)).unwrap();
    assert_eq!(value, json!({"status": "ok", "uptime": 12.35}));
}

#[test]
fn status_omits_an_empty_error() {
    let got = status_from("ok", Some(String::new()), 1.0);
    assert_eq!(got.error, None);
    assert_eq!(serde_json::to_string(&got).unwrap(), r#"{"status":"ok","uptime":1.0}"#);
}

#[test]
fn status_json_shape_with_error() {
    let got = status_from("unhealthy", Some("db down".to_string()), 0.004);
    assert_eq!(
        serde_json::to_string(&got).unwrap(),
        r#"{"status":"unhealthy","uptime":0.0,"error":"db down"}"#
    );
}

#[test]
fn status_rounds_uptime_to_two_decimals() {
    assert_eq!(status_from("ok", None, 7.12345).uptime, 7.12);
    assert_eq!(status_from("ok", None, 2.0051).uptime, 2.01);
}

#[derive(Serialize)]
struct Extended {
    #[serde(flatten)]
    version: Version,
    git_sha: &'static str,
}

#[test]
fn version_flattens_into_a_service_struct() {
    let extended = Extended {
        version: version_from(&env(FULL_ENV), &BUILD),
        git_sha: "fa87ee3",
    };
    assert_eq!(
        serde_json::to_string(&extended).unwrap(),
        r#"{"branch":"main","revision":"fa87ee37083df524ad2ca39826893ce1af482e78","version":"v0.1.1-5-gfa87ee3","git_sha":"fa87ee3"}"#
    );
}
