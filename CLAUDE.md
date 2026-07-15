# CLAUDE.md — bloomctl

Rust CLI for the iru (Kandji) Endpoint Management API. Codegen from a
vendored OpenAPI spec, audit-trail-as-feature, agent-first ergonomics,
read-only against the live tenant by default. Backed by an SDK that
also operates iru's published MCP server as a client.

> **⚠ Tenant data hygiene (load-bearing).** bloomctl operates a live
> MDM fleet. NEVER put PII, device-identifying info (serials, UDIDs,
> names, hostnames), fleet vulnerability details, credentials/secrets
> (tokens, MCP keys, FileVault keys, bypass codes, PINs), unmasked
> tenant/company identity, or raw audit-trail lines into git commits,
> `gh` issues/PRs, discussions, or any shared log. Sanitize first.
> Full rule: `.claude/rules/tenant-data-hygiene.md`.

@charter.md
@.claude/rules/_index.md

## Build / Run / Test

Requires: Rust 1.85+ (Edition 2024), `just`, nightly rustfmt.

```sh
just build              # cargo build --workspace
just test               # cargo test --workspace
just check              # fmt-check + clippy + cargo-deny
just run -- --version   # invoke the CLI
just sync-spec          # cargo xtask sync-spec — refresh vendored OpenAPI
just regen              # cargo xtask regen — rebuild bloomctl-api
```

## Architecture

```
crates/
  bloomctl-api/         Generated reqwest client (regenerable from spec/)
  bloomctl-sdk/         Hand-written: auth chains, audit, redaction,
                        write-guard, spec registry, MCP client, primitives
  bloomctl-cli/         clap CLI: auth/config/ops/api + list/get/search/
                        filter/enrich/emit + the `mcp` subcommand family
  bloomctl-mcp/         Placeholder — MCP *server* backed by bloomctl-sdk

xtask/                  cargo xtask sync-spec | regen | diff-spec
spec/                   Vendored OpenAPI spec (+ sha256 pin)
docs/                   Audit-trail format, design notes
examples/               iru fixtures, jq asserts, recipes
```

Three-layer call graph: `cli/mcp → sdk → api`. Audit emission,
redaction, the write-guard, and the resolution chains live in the SDK
so every consumer inherits them.

## Conventions

- **Language:** Rust, edition 2024, MSRV 1.85.
- **No unsafe:** `#![forbid(unsafe_code)]` everywhere.
- **Generated code:** `bloomctl-api` is rebuilt from `spec/`; do not hand-edit.
- **Auth:** token via env (`BLOOMCTL_API_TOKEN`) → keyring → config;
  subdomain via flag → env (`BLOOMCTL_SUBDOMAIN`) → config. MCP key via
  env → keyring → config. One chain shape, per cli-philosophy.md.
- **Write-guard:** non-GET operations and non-read MCP tools refuse
  without `--allow-write` / `BLOOMCTL_ALLOW_WRITE` / config opt-in.
  The live tenant is production — keep it that way.
- **Audit trail:** every API/MCP call emits a JSONL line under the XDG
  state dir. See `docs/audit-trail-format.md`.
- **No file deletion:** never delete user files. Overwrite only with explicit intent.
- **Git workflow:** trunk-based on `main` until distribution channel exists.

## How to Work Here (kos Process)

### Re-introduction
Read charter.md before any substantive work.

### Session Protocol
1. Read charter.md (orient)
2. Identify the highest-value open question — or capture ideas in `_kos/ideas/`
3. Write an Exploration Brief in `_kos/probes/`
4. Do the probe work
5. Write a finding in `_kos/findings/`
6. Harvest: update affected NODES (`_kos/nodes/{bedrock,frontier,graveyard}/*.yaml`),
   move files if confidence changed. Keep charter edits light per orc
   `.claude/rules/charter-light-touch.md`.

Cross-repo questions belong in the orchestrator's `_kos/`.
