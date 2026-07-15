# Tenant Data Hygiene — Never Publish Fleet-Identifying Data

Behavior-trigger rule. Same shape as the global force-push abort signal
and orc `tooling-friction.md`. Names the *behavior*, not the concept,
because concepts are easy to rationalize past at the keystroke.

bloomctl's entire subject matter is a live, in-use MDM tenant. Every
payload, and most metadata, identifies a real device fleet. A leak here
is not a style problem — it is exposing managed devices, their
vulnerabilities, and their unlock secrets to anyone who reads a public
repo.

## The trigger

You are about to put text into a **durable, shareable channel** — a git
commit message, a file being committed, a `gh issue`/`gh pr` body or
comment, a PR description, a discussion post, a published artifact, or
any log/transcript you paste outward. STOP and scan that text for the
forbidden classes below before it lands.

Also stop when you're about to:
- Paste live `bloomctl` output (stdout OR an audit line) into any of
  the above "to illustrate the bug/feature."
- Regenerate a fixture in `examples/fixtures/` from a real API response.
- Write a test whose expected value is a real device id / serial /
  email / hostname.
- Include a real `--param`/`--where` value in an example or doc.

## Forbidden in any durable/shareable channel — NEVER

1. **PII** — user names, emails, department/title, any person-identifying
   field from `user` / device-assignment records.
2. **Machine-identifying info** — device serials, UDIDs, device names,
   asset tags, MAC addresses, IP addresses, hostnames.
3. **Vulnerability details tied to the fleet** — CVE-to-device mappings,
   per-device exposure, threat/behavioral-detection records. (A CVE id
   in the abstract is fine; "these 3 devices have CVE-X" is a targeting
   map.)
4. **Credentials & secrets** — API tokens, MCP keys (`sk_live:`…), MCP
   profiles, AND secrets-endpoint output: FileVault recovery keys,
   activation-lock bypass codes, recovery-lock passwords, unlock PINs.
   (These are GET endpoints — read-only is NOT non-sensitive.)
5. **Unmasked company/tenant identity** — the tenant subdomain, any
   `*.api.kandji.io` / `*.connect.iru.com` hostname carrying it, the
   org name, internal blueprint/self-service names that name the org.
6. **Raw audit-trail lines** — they carry real path/query params, the
   tenant hostname inside pagination cursors, local hostname/username,
   and CEL predicate text.

## The fix — sanitize before the keystroke

- Prefer **shapes over values**: `ops show <id>`, exit codes, error
  text, `shape_hash` — not payloads.
- Reproduce against **wiremock + `examples/fixtures/`** (synthetic).
  A repro on synthetic data is directly committable as a failing test.
- If real output is unavoidable, substitute: subdomain → `your-subdomain`;
  device id/UDID → `00000000-0000-0000-0000-000000000000`; serial →
  `C02XXXXXXXXX`; name/email → `example`; delete pagination `next` URLs.
- Never share secrets-endpoint output in any form, redacted or not.
- Run shareable repros with `BLOOMCTL_AUDIT=off`.

## Backstops (defense in depth — not a substitute for the rule)

- `scripts/check-tenant-leaks.sh` runs in pre-commit (lefthook) + CI.
  Generic patterns are in-repo; tenant literals go in a **gitignored**
  `.leak-patterns.local` (create one per machine that touches a real
  tenant).
- GitHub secret scanning + push protection are enabled on the repo.
- Fixtures are synthetic **by policy** (`examples/README.md`).

The hooks catch hostnames and key material. They do NOT catch a bare
serial, a device name, or an email — those are on you at the keystroke.
That's why this is a behavior rule, not just a scanner.

## Why this rule exists

The write-guard makes bloomctl safe to *run* against production. This
rule makes bloomctl safe to *develop in the open*. Both are required:
a tool that never mutates the fleet but leaks its inventory to a public
issue tracker has failed the same user. See SECURITY.md "Tenant Data
Hygiene", CONTRIBUTING.md "Sharing logs, payloads, and repros", and the
sanitization checklist baked into the issue templates.

## Cross-references

- Behavior-trigger pattern: `~/.claude/CLAUDE.md` (force-push abort),
  orc `tooling-friction.md`
- SECURITY.md § Tenant Data Hygiene · CONTRIBUTING.md § Sharing logs
- `.github/ISSUE_TEMPLATE/` — the checkbox checklist enforces this on
  every issue
