#!/usr/bin/env bash
# check-tenant-leaks.sh — block tenant-identifying data from entering git.
#
# bloomctl operates against a live MDM tenant. Payloads, audit lines,
# and even hostnames identify the fleet. This check runs generic
# patterns that are safe to publish, plus optional tenant-specific
# literals from a GITIGNORED local file — so the check itself never
# names the tenant.
#
# Usage:
#   scripts/check-tenant-leaks.sh --staged   # pre-commit (staged files)
#   scripts/check-tenant-leaks.sh --all      # CI / full-tree scan
#
# Local extension: create `.leak-patterns.local` (gitignored) with one
# fixed string per line — your tenant subdomain, org name, real device
# serial prefixes, internal hostnames. Lines starting with # are
# comments. Every developer working against a real tenant should
# create one.

set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

MODE="${1:---staged}"

# Placeholder subdomains that are allowed to appear with real host
# suffixes (docs, vendor examples, tests).
PLACEHOLDERS='your-subdomain|your-tenant|subdomain|tenant|example|accuhive|name'

# Generic patterns (ERE). Safe to publish — they describe *shapes*,
# not values.
GENERIC_PATTERNS=(
  # MCP API keys
  'sk_(live|test):[A-Za-z0-9]{16,}'
  # UUID-shaped bearer tokens in pasted commands/output
  'Bearer [0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}'
  # Tenant-shaped API hostnames that are not known placeholders
  'https://[a-z0-9-]{2,}\.api(\.eu)?\.kandji\.io'
  'https://[a-z0-9-]{2,}\.connect\.iru\.com'
)

files() {
  case "$MODE" in
    --staged)
      git diff --cached --name-only --diff-filter=ACM
      ;;
    --all)
      git ls-files
      ;;
    *)
      echo "usage: $0 [--staged|--all]" >&2
      exit 2
      ;;
  esac
}

# Text files only; skip the vendored spec (vendor-published, uses
# placeholder servers) and this script itself.
scan_list() {
  files | grep -vE '^(spec/|target/|scripts/check-tenant-leaks\.sh$)' || true
}

fail=0
matches() {
  # $1 = pattern (ERE), $2.. = files
  local pattern="$1"; shift
  [ "$#" -eq 0 ] && return 0
  grep -nE "$pattern" -- "$@" 2>/dev/null \
    | grep -vE "https://(\{subdomain\}|${PLACEHOLDERS})\." \
    || true
}

FILE_LIST="$(scan_list)"
if [ -z "$FILE_LIST" ]; then
  exit 0
fi

# shellcheck disable=SC2086
for p in "${GENERIC_PATTERNS[@]}"; do
  hits="$(matches "$p" $FILE_LIST)"
  if [ -n "$hits" ]; then
    echo "tenant-leak check: pattern '$p' matched:" >&2
    echo "$hits" >&2
    fail=1
  fi
done

# Tenant-specific literals (gitignored local file — fixed strings).
if [ -f .leak-patterns.local ]; then
  while IFS= read -r needle; do
    case "$needle" in ''|'#'*) continue ;; esac
    # shellcheck disable=SC2086
    hits="$(grep -nF -- "$needle" $FILE_LIST 2>/dev/null || true)"
    if [ -n "$hits" ]; then
      echo "tenant-leak check: local pattern matched (value not echoed):" >&2
      echo "$hits" | cut -d: -f1,2 | sed 's/$/: <redacted match>/' >&2
      fail=1
    fi
  done < .leak-patterns.local
fi

if [ "$fail" -ne 0 ]; then
  cat >&2 <<'EOF'

Tenant-identifying data must not enter git. Sanitize before committing:
  - replace real subdomains/hostnames with your-subdomain
  - replace device ids/serials/emails with synthetic values
  - never commit audit-trail lines or raw API payloads
See SECURITY.md "Tenant Data Hygiene" and CONTRIBUTING.md.
EOF
  exit 1
fi
exit 0
