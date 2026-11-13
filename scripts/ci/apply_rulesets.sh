#!/usr/bin/env bash
# Create or update the repository rulesets defined in .github/rulesets/*.json.
#
# Usage: apply_rulesets.sh [--dry-run] [--repo OWNER/REPO]
#
# Idempotent: a ruleset is matched by its `name`; an existing one is updated in
# place, otherwise it is created. Needs `gh` authenticated as a repo admin.
# Never deletes anything. To switch a ruleset off without removing it, set
# "enforcement": "disabled" in its JSON (or "evaluate" to dry-run its effect)
# and re-run this script.
# @cypher schema=1
# MERGE (self:File {path:"scripts/ci/apply_rulesets.sh"})
# MERGE (main:File {path:".github/rulesets/main.json"})
# MERGE (dev:File {path:".github/rulesets/dev.json"})
# MERGE (gh:ExternalService {id:"gh-cli"})
# MERGE (jq:ExternalService {id:"jq"})
# MERGE (self)-[:READS]->(main)
# MERGE (self)-[:READS]->(dev)
# MERGE (self)-[:CALLS]->(gh)
# MERGE (self)-[:CALLS]->(jq)
# @endcypher
set -euo pipefail

dry_run=false
repo=""
while [ "$#" -gt 0 ]; do
  case "$1" in
    --dry-run) dry_run=true ;;
    --repo) repo="${2:?--repo needs OWNER/REPO}"; shift ;;
    *) echo "usage: $0 [--dry-run] [--repo OWNER/REPO]" >&2; exit 2 ;;
  esac
  shift
done

here="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
repo="${repo:-$(gh repo view --json nameWithOwner --jq .nameWithOwner)}"
echo "repo: $repo (dry-run: $dry_run)"

existing="$(gh api "repos/${repo}/rulesets" --paginate --jq '.[] | "\(.id)\t\(.name)"')"

for file in "$here"/.github/rulesets/*.json; do
  name="$(jq -r .name "$file")"
  id="$(printf '%s\n' "$existing" | awk -F'\t' -v n="$name" '$2==n {print $1; exit}')"
  if [ -n "$id" ]; then
    echo "update  #$id  $name  <- $(basename "$file")"
    [ "$dry_run" = true ] || gh api -X PUT "repos/${repo}/rulesets/${id}" --input "$file" --jq '"  enforcement: \(.enforcement)"'
  else
    echo "create        $name  <- $(basename "$file")"
    [ "$dry_run" = true ] || gh api -X POST "repos/${repo}/rulesets" --input "$file" --jq '"  id: \(.id) enforcement: \(.enforcement)"'
  fi
done
