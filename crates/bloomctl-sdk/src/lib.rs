//! bloomctl-sdk — the SDK that backs bloomctl-cli and (future) bloomctl-mcp.
//!
//! Modules:
//!   * `auth`    — token resolution: env → keyring → config file (env-only in v0.1)
//!   * `audit`   — JSONL audit trail (see `docs/audit-trail-format.md`)
//!   * `client`  — `Client::call_op(operation_id, params)` execution surface
//!   * `error`   — `BloomctlError` + `Result<T>`
//!   * `redact`  — argv + header redaction policy
//!   * `spec`    — operation registry over the vendored OpenAPI spec

#![forbid(unsafe_code)]

pub mod audit;
pub mod auth;
pub mod cel;
pub mod client;
pub mod enrich;
pub mod error;
pub mod kinds;
pub mod mcp;
pub mod redact;
pub mod spec;
pub mod stream;

pub use auth::{ParamSource, ResolvedParam, ResolvedToken, TokenSource};
pub use client::{BASE_URL_ENV, CallOptions, Client};
pub use error::{BloomctlError, Result};
pub use kinds::{KindSpec, all_kinds, extract_items, kind_spec};
pub use mcp::{McpClient, McpCredentials, McpTool};
pub use spec::{HttpMethod, OperationMeta, Registry, registry};
pub use stream::{Record, SourceRef, read_stream, write_record};

pub const SDK_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Channel/build identity stamped by `build.rs`: the CI channel tag
/// (`BLOOMCTL_BUILD_ID` env), a local `dev+g<sha7>[-dirty]`, or
/// `unknown`. Lets audit miners stratify by channel when every channel
/// shares one Cargo version.
pub const BUILD_ID: &str = env!("BLOOMCTL_BUILD_ID");

/// `<semver> (<build_id>)` — the CLI's `--version` string.
pub const FULL_VERSION: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    " (",
    env!("BLOOMCTL_BUILD_ID"),
    ")"
);
