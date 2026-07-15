# Contributing

## Quick start

```sh
just setup           # install nightly rustfmt + cargo-deny
just install-hooks   # lefthook pre-commit + pre-push
just check           # mirror CI quality gates locally
```

## Workflow

- Branch from `main`. Trunk-based until distribution lands.
- Commits follow Conventional Commits. See `.claude/rules/git-commits.md`.
- All commits SSH-signed. CI rejects unsigned commits.
- Open a PR; CI runs fmt, clippy, build, test, cargo-deny, and dependency
  review.

## Sharing logs, payloads, and repros (read before filing issues)

bloomctl runs against live MDM tenants. Anything the tool prints can
identify a real fleet. Before sharing output in an issue, PR, or
commit:

1. **Prefer shapes over values.** Share `bloomctl ops show <id>`,
   exit codes, and error text — not payloads. For response-shape
   questions, share the audit line's `shape_hash` fields, never the
   records.
2. **Reproduce against wiremock or fixtures** where possible
   (`BLOOMCTL_BASE_URL` + `examples/fixtures/`) — repros built on
   synthetic data are directly committable as failing tests.
3. **If real output is unavoidable, sanitize it**: replace the
   subdomain with `your-subdomain`; device ids/UDIDs with
   `00000000-0000-0000-0000-000000000000`; serials with
   `C02XXXXXXXXX`; names/emails with `example` values; delete
   pagination `next` URLs.
4. **Run repros with `BLOOMCTL_AUDIT=off`** if you plan to share your
   terminal transcript, and never share raw audit-trail lines.
5. **Never share secrets-endpoint output** (FileVault keys, bypass
   codes, PINs) in any form, redacted or not.

The pre-commit hook (`scripts/check-tenant-leaks.sh`) blocks
tenant-shaped hostnames and key material; add your tenant's literals
to `.leak-patterns.local` (gitignored) so it also catches your
subdomain, org name, and serial prefixes.

## Spec changes

If the iru OpenAPI spec changes upstream:

```sh
cargo xtask sync-spec         # fetch and update spec/ + sha256
cargo xtask diff-spec         # (forthcoming) summarize the diff
cargo xtask regen             # (forthcoming) regenerate bloomctl-api
```

Open a PR with the spec bump separate from any CLI changes that depend
on it, where reasonable.

## Generated code

`crates/bloomctl-api/` is regenerated from `spec/`. Do not hand-edit
generated source files; modify the spec or the generator config and
regenerate.

## Tests

- Unit tests: `#[cfg(test)] mod tests {}` alongside source.
- HTTP-level tests: `wiremock` against the SDK.
- Snapshot tests: `insta` for stable response renderings.
- CLI smoke tests: `assert_cmd` with a built binary.
