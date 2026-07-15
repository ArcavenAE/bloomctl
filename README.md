# bloomctl

Rust CLI for the [iru](https://www.iru.com/) (formerly Kandji) Endpoint
Management API. Built from a vendored OpenAPI spec, designed for
LLM-driven workflows, with a local audit trail intended to be mined for
the verbs and nouns a v0.2 surface should curate. Sibling of
[sidestep](https://github.com/ArcavenAE/sidestep) — same pattern,
different vendor.

> Status: usable, pending live validation. Auth (env / keyring /
> config), spec-aware operation dispatch, the primitive verbs, the
> write-guard, the MCP client, and the audit trail are all wired and
> tested against a mock server. Live-tenant validation is gated on API
> token permissions.

## Why bloomctl

- **Spec-driven.** `bloomctl-api` is generated from
  `spec/iru-endpoint-openapi.json`. When iru ships new endpoints:
  `cargo xtask sync-spec && cargo xtask regen`, rebuild, ship.
- **SDK-backed.** The same SDK powers the CLI and the MCP-facing
  surfaces. Auth, audit, redaction, and the write-guard live in one
  place.
- **Agent-first.** JSONL output for non-TTY, predictable verb shape,
  stable operation IDs, structured audit trail, CEL predicates.
- **Read-only by default.** The tenant is production. Every non-GET
  operation (and every non-`get-*`/`list-*` MCP tool) is refused
  unless you pass `--allow-write`, set `BLOOMCTL_ALLOW_WRITE=1`, or
  persist `bloomctl config set allow_writes true`.
- **MCP-aware.** iru publishes an MCP server; `bloomctl mcp` operates
  it with the same credential discipline and audit trail — list its
  tools, call them, emit client config, and crosswalk tool names to
  REST operations.
- **Audit as feature.** Every API and MCP call emits a JSONL line
  locally; a future pass mines those traces to propose the composite
  verbs iru workflows actually need.

## Install

```sh
brew tap arcavenae/tap                        # one-time
brew install arcavenae/tap/bloomctl
```

(Formula publishes on first CI release; until then, build from source.)

### Build from source

```sh
git clone https://github.com/ArcavenAE/bloomctl.git
cd bloomctl
cargo build --release
./target/release/bloomctl --version
```

## Getting started

bloomctl needs two values: an **API token** and your **tenant
subdomain** (the `<name>` in `https://<name>.api.kandji.io` — create
tokens in iru Access, scope their permissions there).

```sh
# Store the token in the platform keyring + persist the subdomain:
echo "$YOUR_IRU_TOKEN" | bloomctl auth login --stdin --subdomain your-tenant

# Verify (prints sources + lengths, never secrets):
bloomctl auth status
```

Environment variables take precedence over stored values:
`BLOOMCTL_API_TOKEN`, `BLOOMCTL_SUBDOMAIN`, `BLOOMCTL_REGION` (`us` |
`eu`). A config file at `~/.config/bloomctl/config.toml` (override
path with `BLOOMCTL_CONFIG`) is the final fallback. Resolution order
is **flag → env → keyring/config → error**, and every error names the
full chain.

## Quick verification

```sh
# 1. Discover what operations are available.
bloomctl ops list | head
bloomctl ops list --filter device

# 2. Inspect one operation's path, params, and write-guard status.
bloomctl ops show get_devices

# 3. Make a real read-only call.
bloomctl api get_devices --param limit=1

# 4. Or use the primitive verbs.
bloomctl list device --limit 5 | bloomctl emit --format md
```

Every call writes an audit line under `~/.local/state/bloomctl/audit/`
(Linux) or the platform state dir (macOS). The line records the
operation, params, response shape hash, status, duration, and where
each credential was resolved from — never the secrets themselves.

## Usage

```sh
bloomctl auth login --stdin --subdomain <name>   # keyring + config
bloomctl auth status                             # sources, never secrets
bloomctl auth logout

bloomctl ops list [--filter <substring>]         # spec discovery
bloomctl ops show <operationId>

bloomctl api <operationId> [--param k=v ...] [--body '<json>']
    [--subdomain <name>] [--allow-write] [--no-audit]

# Primitives — compose over _kind-tagged JSONL streams:
bloomctl list <kind> [--param k=v] [--limit N] [--since 24h]
bloomctl get <kind> <id>
bloomctl search <kind> <text>
bloomctl filter --where '<CEL>' [--explain]
bloomctl enrich --with <recipe> [--blueprints FILE]
bloomctl emit [--format jsonl|md]

# MCP — operate iru's published MCP server:
bloomctl mcp login --stdin --profile <hex>       # sk_live key → keyring
bloomctl mcp status
bloomctl mcp config [--reveal]                   # client config JSON
bloomctl mcp tools [--filter <substring>]
bloomctl mcp call <tool> [--args '<json>'] [--allow-write]
bloomctl mcp map                                 # tool ↔ operationId crosswalk

bloomctl config show|path|set|unset              # persisted defaults
```

v0.1 kinds: `device`, `blueprint`, `user`, `tag`, `audit_event`,
`threat`, `behavioral_detection`, `vulnerability`, `custom_app`,
`custom_profile`, `custom_script`, `ade_device`.

Recipes in [`examples/recipes/`](examples/recipes/) show composed
flows (fleet inventory, stale devices, vulnerability triage).

## The write-guard

iru tenants are living fleets. bloomctl refuses anything mutating —
POST/PATCH/DELETE operations and non-read MCP tools — unless you opt
in explicitly:

```sh
bloomctl api patch_devices_device_id --param device_id=<uuid> \
    --body '{"asset_tag":"A-100"}' --allow-write
```

Standing opt-ins (`BLOOMCTL_ALLOW_WRITE=1` env, `allow_writes = true`
config) exist for automation that has earned them; `auth status`
reports the posture loudly.

## Development

```sh
just build           # cargo build --workspace
just test            # cargo test --workspace --all-targets
just check           # fmt + clippy + cargo-deny
just sync-spec       # cargo xtask sync-spec — refresh vendored OpenAPI
just regen           # cargo xtask regen — rebuild bloomctl-api
make -C examples assert   # jq-simulated regression asserts
```

See [CLAUDE.md](CLAUDE.md) and [charter.md](charter.md) for design
context, and [docs/audit-trail-format.md](docs/audit-trail-format.md)
for the audit-trail JSONL schema.

## License

MIT — see [LICENSE](LICENSE).
