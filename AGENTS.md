# AGENTS.md

Orientation for AI agents working in this repo.

## What this repo is

The `tatari-service-lib` crate: a Rust port of `tatari_service_lib.routes` and `extractors` from `tatari-tv/python-tatari-service-lib`. It answers Tatari's standard routes, `/version`, `/deployed` and `/status`, as plain functions returning serde structs, read from the runtime env the platform injects. `src/lib.rs` holds `Build`, `init` and the process start; `src/env.rs` holds the extractors (`usable`, `environment`, `deployed_at`); `src/routes.rs` holds the three route bodies and their structs. Design: `tatari-tv/rust` `docs/design/2026-10-30-service-golden-path.md`.

## Rules

- No web framework dependency, ever. `cargo tree -e normal --prefix none | rg -w 'axum|actix-web|rocket'` must print nothing.
- Every env read goes through an injected `Fn(&str) -> Option<String>`. Tests pass a map (`src/fake.rs`) and never call `set_var`.
- An empty or literal `unknown` env value counts as unset; route every read through `env::usable`.
- Optional keys are `Option` with `skip_serializing_if`, so a service can `#[serde(flatten)]` the structs.
- Tags are flat `vX.Y.Z`, annotated, on `main` only; consumers pin the git tag.
