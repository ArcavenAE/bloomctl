#!/usr/bin/env bash
# Recipe: fleet inventory — devices with blueprint context, as a table.
#
#   bloomctl list blueprints > /tmp/blueprints.jsonl
#   bloomctl list devices \
#     | bloomctl enrich --with blueprint-context --blueprints /tmp/blueprints.jsonl \
#     | bloomctl emit --format md
#
# Requires: bloomctl auth login (token + subdomain) with read permissions.

set -euo pipefail

tmp="$(mktemp)"
trap 'rm -f "$tmp"' EXIT

bloomctl list blueprint > "$tmp"
bloomctl list device \
  | bloomctl enrich --with blueprint-context --blueprints "$tmp" \
  | bloomctl emit --format md
