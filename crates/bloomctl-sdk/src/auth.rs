//! Token, subdomain, and MCP-credential resolution chains.
//!
//! All chains are instantiations of the `val-resolution-chain` bedrock
//! pattern (see charter.md B5 and `.claude/rules/cli-philosophy.md`):
//!
//! * **Token chain** — env → keyring → config file → error.
//! * **Subdomain chain** — flag → env → config file → error. The iru
//!   tenant subdomain is part of the API *hostname*
//!   (`https://<subdomain>.api.kandji.io`), constant for the lifetime
//!   of a token — the abusive-argument test applies, so it resolves
//!   through a chain instead of being a required flag.
//! * **MCP chains** — the MCP API key (env → keyring → config) and
//!   MCP profile (env → config) for iru's published MCP server.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::{BloomctlError, Result};

pub const TOKEN_ENV: &str = "BLOOMCTL_API_TOKEN";
pub const SUBDOMAIN_ENV: &str = "BLOOMCTL_SUBDOMAIN";
pub const REGION_ENV: &str = "BLOOMCTL_REGION";
pub const ALLOW_WRITE_ENV: &str = "BLOOMCTL_ALLOW_WRITE";
pub const CONFIG_ENV: &str = "BLOOMCTL_CONFIG";
pub const MCP_API_KEY_ENV: &str = "BLOOMCTL_MCP_API_KEY";
pub const MCP_PROFILE_ENV: &str = "BLOOMCTL_MCP_PROFILE";
pub const MCP_URL_ENV: &str = "BLOOMCTL_MCP_URL";
pub const KEYRING_SERVICE: &str = "bloomctl";
pub const KEYRING_USER: &str = "default";
pub const KEYRING_MCP_USER: &str = "mcp-api-key";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TokenSource {
    Env,
    Keyring,
    Config,
}

impl TokenSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            TokenSource::Env => "env",
            TokenSource::Keyring => "keyring",
            TokenSource::Config => "config",
        }
    }
}

/// Source of a chain-resolved non-secret value. Mirrors `TokenSource`
/// but drops `Keyring` (non-secrets don't live there) and adds a
/// `Flag` variant for explicit per-call overrides.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ParamSource {
    Flag,
    Env,
    Config,
}

impl ParamSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            ParamSource::Flag => "flag",
            ParamSource::Env => "env",
            ParamSource::Config => "config",
        }
    }
}

#[derive(Clone, Debug)]
pub struct ResolvedToken {
    pub token: String,
    pub source: TokenSource,
}

#[derive(Clone, Debug)]
pub struct ResolvedParam {
    pub value: String,
    pub source: ParamSource,
}

/// Resolve the REST API token by walking the precedence chain:
/// env → keyring → config file → error.
///
/// Keyring backend errors (no daemon, denied access) fall through to the
/// next layer rather than surfacing as fatal — so a user with only env
/// or only a config file isn't blocked by a missing Secret Service.
/// A malformed config file IS fatal: silent failure here would mask a
/// real auth misconfiguration.
pub fn resolve() -> Result<ResolvedToken> {
    if let Ok(t) = std::env::var(TOKEN_ENV) {
        if !t.is_empty() {
            return Ok(ResolvedToken {
                token: t,
                source: TokenSource::Env,
            });
        }
    }
    if let Some(t) = read_keyring() {
        return Ok(ResolvedToken {
            token: t,
            source: TokenSource::Keyring,
        });
    }
    if let Some(t) = read_config_token()? {
        return Ok(ResolvedToken {
            token: t,
            source: TokenSource::Config,
        });
    }
    Err(BloomctlError::Auth(format!(
        "no token found. Set {TOKEN_ENV}=<api-token>, run \
         `bloomctl auth login --token <api-token>` to store one in \
         the platform keyring, or write `[auth] token = \"<value>\"` \
         to {}.",
        config_path()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "~/.config/bloomctl/config.toml".into())
    )))
}

/// Backwards-compatible single-string return; prefer `resolve` so the
/// caller can record `TokenSource` in audit metadata.
pub fn resolve_token() -> Result<String> {
    resolve().map(|r| r.token)
}

/// Resolve the tenant subdomain by walking flag → env
/// (`BLOOMCTL_SUBDOMAIN`) → config (`[default] subdomain`). Returns
/// `Ok(None)` when no source supplies a value — the caller (usually
/// `Client` construction) raises an error naming every layer of the
/// chain.
pub fn resolve_subdomain(flag: Option<&str>) -> Result<Option<ResolvedParam>> {
    resolve_param(flag, SUBDOMAIN_ENV, |c| c.default.subdomain.clone())
}

/// Resolve the API region (`us` or `eu`) by walking flag → env
/// (`BLOOMCTL_REGION`) → config (`[default] region`). Returns
/// `Ok(None)` when unset — callers default to `us`.
pub fn resolve_region(flag: Option<&str>) -> Result<Option<ResolvedParam>> {
    resolve_param(flag, REGION_ENV, |c| c.default.region.clone())
}

/// True when write operations are allowed without a per-call
/// `--allow-write`. Walks env (`BLOOMCTL_ALLOW_WRITE`, any of
/// `1`/`true`/`yes`) → config (`[default] allow_writes = true`).
pub fn writes_allowed_by_default() -> Result<bool> {
    if let Ok(v) = std::env::var(ALLOW_WRITE_ENV) {
        let v = v.trim().to_ascii_lowercase();
        if v == "1" || v == "true" || v == "yes" {
            return Ok(true);
        }
    }
    Ok(read_config()?
        .map(|c| c.default.allow_writes.unwrap_or(false))
        .unwrap_or(false))
}

/// Resolve the MCP API key (the `sk_live:`-prefixed value from the iru
/// MCP configuration) by walking env → keyring → config `[mcp] api_key`.
pub fn resolve_mcp_api_key() -> Result<Option<ResolvedToken>> {
    if let Ok(t) = std::env::var(MCP_API_KEY_ENV) {
        if !t.is_empty() {
            return Ok(Some(ResolvedToken {
                token: t,
                source: TokenSource::Env,
            }));
        }
    }
    if let Some(t) = read_keyring_entry(KEYRING_MCP_USER) {
        return Ok(Some(ResolvedToken {
            token: t,
            source: TokenSource::Keyring,
        }));
    }
    if let Some(t) = read_config()?.and_then(|c| c.mcp.api_key.filter(|t| !t.is_empty())) {
        return Ok(Some(ResolvedToken {
            token: t,
            source: TokenSource::Config,
        }));
    }
    Ok(None)
}

/// Resolve the MCP profile identifier by walking env → config
/// (`[mcp] profile`). Not a secret, so no keyring layer.
pub fn resolve_mcp_profile() -> Result<Option<ResolvedParam>> {
    resolve_param(None, MCP_PROFILE_ENV, |c| c.mcp.profile.clone())
}

/// Resolve the MCP server URL by walking env → config (`[mcp] url`).
/// Returns `Ok(None)` when unset — callers derive the default from the
/// subdomain chain (`https://<subdomain>.connect.iru.com/...`).
pub fn resolve_mcp_url() -> Result<Option<ResolvedParam>> {
    resolve_param(None, MCP_URL_ENV, |c| c.mcp.url.clone())
}

fn resolve_param(
    flag: Option<&str>,
    env_var: &str,
    from_config: impl FnOnce(&Config) -> Option<String>,
) -> Result<Option<ResolvedParam>> {
    if let Some(v) = flag {
        let trimmed = v.trim();
        if !trimmed.is_empty() {
            return Ok(Some(ResolvedParam {
                value: trimmed.to_string(),
                source: ParamSource::Flag,
            }));
        }
    }
    if let Ok(v) = std::env::var(env_var) {
        let trimmed = v.trim();
        if !trimmed.is_empty() {
            return Ok(Some(ResolvedParam {
                value: trimmed.to_string(),
                source: ParamSource::Env,
            }));
        }
    }
    if let Some(cfg) = read_config()? {
        if let Some(v) = from_config(&cfg) {
            let trimmed = v.trim();
            if !trimmed.is_empty() {
                return Ok(Some(ResolvedParam {
                    value: trimmed.to_string(),
                    source: ParamSource::Config,
                }));
            }
        }
    }
    Ok(None)
}

/// Read the API token from the platform keyring. Returns `None` for
/// both "no entry" and "backend unavailable" — both mean "fall through."
pub fn read_keyring() -> Option<String> {
    read_keyring_entry(KEYRING_USER)
}

fn read_keyring_entry(user: &str) -> Option<String> {
    let entry = keyring::Entry::new(KEYRING_SERVICE, user).ok()?;
    entry.get_password().ok()
}

/// Store the API token in the platform keyring, replacing any existing
/// entry.
pub fn store_keyring(token: &str) -> Result<()> {
    store_keyring_entry(KEYRING_USER, token)
}

/// Store the MCP API key in the platform keyring under the dedicated
/// `mcp-api-key` user.
pub fn store_mcp_keyring(key: &str) -> Result<()> {
    store_keyring_entry(KEYRING_MCP_USER, key)
}

/// Read the MCP API key from the keyring (`None` = absent/unavailable).
pub fn read_mcp_keyring() -> Option<String> {
    read_keyring_entry(KEYRING_MCP_USER)
}

fn store_keyring_entry(user: &str, secret: &str) -> Result<()> {
    if secret.is_empty() {
        return Err(BloomctlError::Auth("value must not be empty".into()));
    }
    let entry = keyring::Entry::new(KEYRING_SERVICE, user)
        .map_err(|e| BloomctlError::Auth(format!("keyring open: {e}")))?;
    entry
        .set_password(secret)
        .map_err(|e| BloomctlError::Auth(format!("keyring write: {e}")))?;
    Ok(())
}

/// Delete the API-token keyring entry. Returns `Ok(false)` if there was
/// nothing to delete — that is not an error.
pub fn delete_keyring() -> Result<bool> {
    delete_keyring_entry(KEYRING_USER)
}

/// Delete the MCP API-key keyring entry (`Ok(false)` = nothing there).
pub fn delete_mcp_keyring() -> Result<bool> {
    delete_keyring_entry(KEYRING_MCP_USER)
}

fn delete_keyring_entry(user: &str) -> Result<bool> {
    let entry = keyring::Entry::new(KEYRING_SERVICE, user)
        .map_err(|e| BloomctlError::Auth(format!("keyring open: {e}")))?;
    match entry.delete_credential() {
        Ok(()) => Ok(true),
        Err(keyring::Error::NoEntry) => Ok(false),
        Err(e) => Err(BloomctlError::Auth(format!("keyring delete: {e}"))),
    }
}

/// Configuration loaded from `~/.config/bloomctl/config.toml` (or the
/// path in `BLOOMCTL_CONFIG`). The struct is `#[serde(default)]` so
/// future sections can be added without breaking parsing.
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Config {
    #[serde(skip_serializing_if = "AuthConfig::is_empty")]
    pub auth: AuthConfig,
    #[serde(skip_serializing_if = "DefaultConfig::is_empty")]
    pub default: DefaultConfig,
    #[serde(skip_serializing_if = "McpConfig::is_empty")]
    pub mcp: McpConfig,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct AuthConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
}

impl AuthConfig {
    fn is_empty(&self) -> bool {
        self.token.is_none()
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct DefaultConfig {
    /// Tenant subdomain — the `<subdomain>` in
    /// `https://<subdomain>.api.kandji.io`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subdomain: Option<String>,
    /// API region: `us` (default) or `eu`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
    /// Standing opt-in for write (non-GET) operations. Defaults to
    /// false — bloomctl is read-only against the tenant unless the
    /// caller passes `--allow-write` or sets this.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_writes: Option<bool>,
}

impl DefaultConfig {
    fn is_empty(&self) -> bool {
        self.subdomain.is_none() && self.region.is_none() && self.allow_writes.is_none()
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct McpConfig {
    /// MCP API key (`sk_live:` prefixed). Prefer the keyring
    /// (`bloomctl mcp login`) over storing this in plain TOML.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    /// MCP profile identifier (`X-MCP-Profile` header value).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<String>,
    /// Full MCP server URL override. When unset, derived from the
    /// subdomain: `https://<subdomain>.connect.iru.com/mcp-server/connector/kandji/tools`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

impl McpConfig {
    fn is_empty(&self) -> bool {
        self.api_key.is_none() && self.profile.is_none() && self.url.is_none()
    }
}

/// Resolve the config file path. `BLOOMCTL_CONFIG` overrides; otherwise
/// the XDG config dir + `bloomctl/config.toml` is used. Returns `None`
/// only when neither override nor a home directory is discoverable.
pub fn config_path() -> Option<PathBuf> {
    if let Ok(p) = std::env::var(CONFIG_ENV) {
        if !p.is_empty() {
            return Some(PathBuf::from(p));
        }
    }
    dirs::config_dir().map(|d| d.join("bloomctl").join("config.toml"))
}

/// Read and parse the config file. Returns:
///   * `Ok(Some(cfg))` — file present, parsed
///   * `Ok(None)`      — file absent, or no discoverable config path
///   * `Err(...)`      — file present but malformed (TOML parse failed)
pub fn read_config() -> Result<Option<Config>> {
    let Some(path) = config_path() else {
        return Ok(None);
    };
    let body = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => {
            return Err(BloomctlError::Auth(format!(
                "read config {}: {e}",
                path.display()
            )));
        }
    };
    let parsed: Config = toml::from_str(&body)
        .map_err(|e| BloomctlError::Auth(format!("parse config {}: {e}", path.display())))?;
    Ok(Some(parsed))
}

/// Read the token from the config file. Returns:
///   * `Ok(Some(token))` — file present, parsed, token non-empty
///   * `Ok(None)`        — file absent, or present but no `[auth].token`
///   * `Err(...)`        — file present but malformed (TOML parse failed)
pub fn read_config_token() -> Result<Option<String>> {
    Ok(read_config()?.and_then(|c| c.auth.token.filter(|t| !t.is_empty())))
}

/// Read-merge-write the config file. Loads the existing config (or a
/// fresh default), applies `mutate`, then writes the result to disk —
/// preserving every section the caller did not touch. Creates the
/// parent directory if missing. Returns the path that was written.
pub fn write_config(mutate: impl FnOnce(&mut Config)) -> Result<PathBuf> {
    let path =
        config_path().ok_or_else(|| BloomctlError::Auth("no discoverable config path".into()))?;
    let mut cfg = read_config()?.unwrap_or_default();
    mutate(&mut cfg);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            BloomctlError::Auth(format!("create config dir {}: {e}", parent.display()))
        })?;
    }
    let body = toml::to_string_pretty(&cfg)
        .map_err(|e| BloomctlError::Auth(format!("serialize config: {e}")))?;
    std::fs::write(&path, body)
        .map_err(|e| BloomctlError::Auth(format!("write config {}: {e}", path.display())))?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_source_as_str_is_stable() {
        assert_eq!(TokenSource::Env.as_str(), "env");
        assert_eq!(TokenSource::Keyring.as_str(), "keyring");
        assert_eq!(TokenSource::Config.as_str(), "config");
    }

    #[test]
    fn param_source_as_str_is_stable() {
        assert_eq!(ParamSource::Flag.as_str(), "flag");
        assert_eq!(ParamSource::Env.as_str(), "env");
        assert_eq!(ParamSource::Config.as_str(), "config");
    }

    #[test]
    fn parse_full_config() {
        let body = r#"
[auth]
token = "abc123"

[default]
subdomain = "accuhive"
region = "us"
allow_writes = false

[mcp]
profile = "cafe0123"
"#;
        let cfg: Config = toml::from_str(body).expect("parse");
        assert_eq!(cfg.auth.token.as_deref(), Some("abc123"));
        assert_eq!(cfg.default.subdomain.as_deref(), Some("accuhive"));
        assert_eq!(cfg.default.region.as_deref(), Some("us"));
        assert_eq!(cfg.default.allow_writes, Some(false));
        assert_eq!(cfg.mcp.profile.as_deref(), Some("cafe0123"));
    }

    #[test]
    fn parse_empty_config() {
        let cfg: Config = toml::from_str("").expect("parse");
        assert_eq!(cfg.auth.token, None);
        assert_eq!(cfg.default.subdomain, None);
        assert_eq!(cfg.mcp.profile, None);
    }

    #[test]
    fn parse_unrelated_sections_ok() {
        // Future sections must not break parsing of known sections.
        let body = r#"
[future_section]
flag = true

[auth]
token = "xyz"

[default]
subdomain = "accuhive"
"#;
        let cfg: Config = toml::from_str(body).expect("parse");
        assert_eq!(cfg.auth.token.as_deref(), Some("xyz"));
        assert_eq!(cfg.default.subdomain.as_deref(), Some("accuhive"));
    }

    #[test]
    fn parse_default_only_no_auth() {
        let body = r#"
[default]
subdomain = "accuhive"
"#;
        let cfg: Config = toml::from_str(body).expect("parse");
        assert_eq!(cfg.auth.token, None);
        assert_eq!(cfg.default.subdomain.as_deref(), Some("accuhive"));
    }

    #[test]
    fn parse_malformed_errors() {
        let body = "this is = not = toml";
        let result: std::result::Result<Config, _> = toml::from_str(body);
        assert!(result.is_err());
    }

    #[test]
    fn serialize_skips_empty_sections() {
        let cfg = Config::default();
        let body = toml::to_string_pretty(&cfg).expect("serialize");
        assert!(
            !body.contains("[auth]"),
            "empty auth section should be skipped: {body:?}"
        );
        assert!(
            !body.contains("[default]"),
            "empty default section should be skipped: {body:?}"
        );
        assert!(
            !body.contains("[mcp]"),
            "empty mcp section should be skipped: {body:?}"
        );
    }

    #[test]
    fn serialize_round_trip_default_only() {
        let mut cfg = Config::default();
        cfg.default.subdomain = Some("accuhive".into());
        let body = toml::to_string_pretty(&cfg).expect("serialize");
        assert!(body.contains("[default]"), "want [default] in {body:?}");
        assert!(
            body.contains("subdomain = \"accuhive\""),
            "want subdomain in {body:?}"
        );
        assert!(
            !body.contains("[auth]"),
            "empty auth section should be skipped: {body:?}"
        );
    }
}
