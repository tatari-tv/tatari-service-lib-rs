# tatari-service-lib-rs

Tatari's standard service routes, `/version`, `/deployed` and `/status`, for Rust services. A port of `tatari_service_lib.routes` and `extractors` from [python-tatari-service-lib](https://github.com/tatari-tv/python-tatari-service-lib), so a Rust service answers the same contract a Python one does.

The crate is plain functions returning serde structs. It has no web framework dependency: your axum, actix or rocket handler wraps the struct in its own JSON response.

Tracked under [SRE-4431](https://tatari.atlassian.net/browse/SRE-4431).

## The contract

Every value is read at runtime from the env the platform injects. The image env (`GIT_*`) comes from the shared `docker-build` action and the base image; the chart env (`ENV`, `DEPLOY_DATETIME`) comes from the `platform-service` Helm chart.

| route | key | env | fallback |
| --- | --- | --- | --- |
| `/version` | `branch` | `GIT_BRANCH` | `Build.branch` |
| `/version` | `revision` | `GIT_VERSION` | `Build.revision` |
| `/version` | `version` | `GIT_DESCRIBE` | `Build.describe` |
| `/deployed` | `deployed_at` | `DEPLOY_DATETIME`, RFC 3339 or the comma-fraction form, output as RFC 3339 UTC | process start; unparseable logs a warning and uses process start |
| `/deployed` | `deployments_link` | `DEPLOYMENTS_LINK` | key omitted |
| `/deployed` | `environment` | `ENV`, lowercased | `ci` when `CI` is set, else `local` |
| `/deployed` | `deployer` | `GIT_AUTHOR` | `""`; key omitted when `privacy` is true |
| `/status` | `status`, `uptime` (seconds, 2 dp), `error` (only when set and non-empty) | process | |

An env value that is empty or the literal `unknown` (the base image's default for every `GIT_*`) counts as unset.

Two differences from the Python lib, both on purpose:

- `/version` falls back to the binary's compile-time `Build` instead of `""`, so a laptop `cargo run` still reports a true version.
- `deployments_link` is emitted only when `DEPLOYMENTS_LINK` is set. No pod sets it today, so the key would always be `""`.

## Usage

Depend on it by git tag:

```toml
[dependencies]
tatari-service-lib = { git = "https://github.com/tatari-tv/tatari-service-lib-rs", tag = "<tag>" }
```

Your binary's `build.rs` emits `GIT_BRANCH`, `GIT_REVISION` and `GIT_DESCRIBE` as compile-time env (persona's `server/build.rs` is the canonical one). Pass them in as `Build`, call `init()` first thing in `main`, and wire the three routes in your framework. With axum:

```rust
use axum::{Json, Router, routing::get};
use tatari_service_lib::{Build, Deployed, Status, Version};

const BUILD: Build = Build {
    branch: env!("GIT_BRANCH"),
    revision: env!("GIT_REVISION"),
    describe: env!("GIT_DESCRIBE"),
};

async fn version() -> Json<Version> {
    Json(tatari_service_lib::version(&BUILD))
}

async fn deployed() -> Json<Deployed> {
    Json(tatari_service_lib::deployed(false))
}

async fn status() -> Json<Status> {
    Json(tatari_service_lib::status("ok", None))
}

#[tokio::main]
async fn main() {
    tatari_service_lib::init();
    let app = Router::new()
        .route("/version", get(version))
        .route("/deployed", get(deployed))
        .route("/status", get(status));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

A service that answers extra keys wraps the struct with `#[serde(flatten)]`:

```rust
#[derive(serde::Serialize)]
struct VersionWithSha {
    #[serde(flatten)]
    version: tatari_service_lib::Version,
    git_sha: &'static str,
}
```

`/health` and `/ready` are not here: they are Kubernetes probe semantics each service owns.

## Development

`otto ci` runs lint, clippy, fmt and the tests. Tests inject the env through a lookup function and never mutate the process env.
