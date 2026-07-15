#!/usr/bin/env bash
# Recipe: stale devices — anything that hasn't checked in for 30 days.
#
# The canonical fleet-health question, composed from primitives:
# list → CEL filter on the promoted last_check_in timestamp → table.

set -euo pipefail

bloomctl list device \
  | bloomctl filter --where 'last_check_in < now - duration("720h")' \
  | bloomctl emit --format md
