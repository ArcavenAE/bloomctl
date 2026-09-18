# finding-001: keyring v3 to v4 migration (replay of sidestep finding-010)

Status: replay. The reference is sidestep `_kos/findings/finding-010-keyring-v3-to-v4-crate-split-migration.md`, which records why v4 is a crate split (not a feature rename) and why the v1 facade is the remedy. bloomctl is the second of three sibling CLIs (sidestep, bloomctl, stave) to make this move; it replays the reference rather than rediscover it.

Date: 2026-09-18. bd: `aae-orc-vch2t`. Plan of record:
`director/_bmad-output/keyring-v3-v4-migration-plan-2026-09-18.md`.

## The change

One line in the workspace-root `Cargo.toml`:

```
-keyring = { version = "3", features = ["apple-native", "linux-native"] }
+keyring = "4"
```

Drop the explicit feature list; keyring 4's default `v1` feature supplies the
macOS Keychain backend. The SDK crate's `keyring.workspace = true` needs no
edit. `cargo update -p keyring` regenerates the lockfile.

## Result: no auth-module code change

bloomctl uses the same simple keyring surface the reference covers, across two
accounts under the `bloomctl` service (`default` for the API token and
`mcp-api-key` for the MCP key): `keyring::Entry::new(SERVICE, user)`,
`set_password`, `get_password`, `delete_credential`, and one
`Err(keyring::Error::NoEntry)` arm (`crates/bloomctl-sdk/src/auth.rs`). All are
unchanged in the v1 facade. The migration compiled with zero source edits.

Green on this host (macOS, Apple Silicon), from a Cargo.toml + Cargo.lock change
only:

- `cargo build --workspace` clean
- `cargo clippy --workspace --all-targets -- -D warnings` clean
- `cargo test --workspace`: 149 passed, 0 failed
- `cargo deny check`: advisories, bans, licenses, sources all ok

## Resolved dependency shape (host: macOS)

`cargo update -p keyring` moved v3.6.3 to v4.2.0, added keyring-core v1.0.0 and
apple-native-keyring-store v1.0.2, and removed the v3-era security-framework
2.11.1 and linux-keyutils. `cargo tree -p keyring` confirms
apple-native-keyring-store is the active macOS backend.

## Local Keychain round-trip: verified

keyring v4, under a throwaway service (`bloomctl-keyring-migration-test`) so the
real `bloomctl` entries are untouched, in a single process: `Entry::new` ->
`set_password` -> `get_password` (read-back equal) -> `delete_credential` ->
`get_password` returns `Err(NoEntry)`. Passed; no leftover keychain item; no
prompt.

## Cross-major read-back (the one thing docs cannot answer)

Same open question as the reference: does a Keychain item written by v3's
`apple-native` read back under v4's `apple-native-keyring-store`. Measured by the
director's read-only smoke test on the built v4 binary against bloomctl's own
stored credential. bloomctl holds a different vendor credential than sidestep, so
it is measured on its own. The agent does not touch the live vendor tenant.

RESULT: pending the director's read-only read-back on the built v4 bloomctl
binary. Update this line once measured.

## Linux caveat (recorded, not chased)

Path A swaps the Linux backend from `linux-keyutils` to the D-Bus secret service,
as in the reference. Moot for a macOS fleet with Linux compile-only validation.

## Sources

Reference finding sidestep finding-010 (crate-split model, v1-facade remedy,
byte-verified upstream sources). No new source work was needed for the replay.
