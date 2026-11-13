#!/usr/bin/env bash
# Dispatch the stable-release workflows for one tag, wait for them, and fail if
# any of them fails.
#
# Usage: dispatch_release_workflows.sh <tag> <components>
#   components: all | desktop | web | canvas   (comma separated also accepted)
#
# Requires: gh (authenticated via GH_TOKEN with `actions: write`), jq.
#
# This script does NOT bypass the release gates: each target workflow still
# requires its own APPROVED input, a `refs/tags/v*` ref, and (where configured)
# the protected `star-stable-release` environment. It only removes the need to
# click three separate "Run workflow" forms for the same tag.
# @cypher schema=1
# MERGE (self:File {path:"scripts/ci/dispatch_release_workflows.sh"})
# MERGE (desktop:File {path:".github/workflows/star-stable-desktop.yml"})
# MERGE (web:File {path:".github/workflows/star-stable-web.yml"})
# MERGE (canvas:File {path:".github/workflows/publish-star-canvas-services.yml"})
# MERGE (dispatch_one:Symbol {id:"scripts/ci/dispatch_release_workflows.sh::dispatch_one",kind:"function"})
# MERGE (wait_one:Symbol {id:"scripts/ci/dispatch_release_workflows.sh::wait_one",kind:"function"})
# MERGE (gh:ExternalService {id:"gh-cli"})
# MERGE (self)-[:DEFINES]->(dispatch_one)
# MERGE (self)-[:DEFINES]->(wait_one)
# MERGE (dispatch_one)-[:CALLS]->(gh)
# MERGE (wait_one)-[:CALLS]->(gh)
# MERGE (dispatch_one)-[:CALLS]->(desktop)
# MERGE (dispatch_one)-[:CALLS]->(web)
# MERGE (dispatch_one)-[:CALLS]->(canvas)
# @endcypher
set -euo pipefail

if [ "$#" -ne 2 ]; then
  echo "usage: $0 <tag> <components>" >&2
  exit 2
fi
tag="$1"
components="$2"

declare -A WORKFLOW=(
  [desktop]="star-stable-desktop.yml"
  [web]="star-stable-web.yml"
  [canvas]="publish-star-canvas-services.yml"
)
# The three workflows name their confirmation input differently.
declare -A INPUT=(
  [desktop]="release_confirmation"
  [web]="release_gate_confirmation"
  [canvas]="release_gate_confirmation"
)

if [ "$components" = "all" ]; then
  selected=(desktop web canvas)
else
  IFS=',' read -r -a selected <<<"$components"
fi
for c in "${selected[@]}"; do
  [ -n "${WORKFLOW[$c]:-}" ] || { echo "::error::unknown component '$c'" >&2; exit 2; }
done

declare -A RUN_ID

dispatch_one() {
  local component="$1" file input started id=""
  file="${WORKFLOW[$component]}"
  input="${INPUT[$component]}"
  started="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "dispatching $file on $tag"
  gh workflow run "$file" --ref "$tag" -f "${input}=APPROVED"
  # The dispatch API returns no run id; find the run created after `started`.
  for _ in $(seq 1 30); do
    id="$(gh run list --workflow "$file" --event workflow_dispatch --branch "$tag" \
      --json databaseId,createdAt \
      --jq "[.[] | select(.createdAt >= \"${started}\")] | sort_by(.createdAt) | last | .databaseId // empty")"
    [ -n "$id" ] && break
    sleep 10
  done
  if [ -z "$id" ]; then
    echo "::error::could not find the dispatched run of $file within 5 minutes" >&2
    return 1
  fi
  RUN_ID[$component]="$id"
  echo "$component -> https://github.com/${GITHUB_REPOSITORY}/actions/runs/${id}"
}

wait_one() {
  local component="$1"
  gh run watch "${RUN_ID[$component]}" --exit-status --interval 30 >/dev/null
}

for c in "${selected[@]}"; do
  dispatch_one "$c"
done

failed=0
for c in "${selected[@]}"; do
  if wait_one "$c"; then
    echo "ok   $c (run ${RUN_ID[$c]})"
  else
    echo "FAIL $c (run ${RUN_ID[$c]})"
    failed=1
  fi
done

# Machine-readable list for the release-notes step.
if [ -n "${GITHUB_OUTPUT:-}" ]; then
  {
    echo "runs<<EOF"
    for c in "${selected[@]}"; do
      echo "- ${c}: https://github.com/${GITHUB_REPOSITORY}/actions/runs/${RUN_ID[$c]}"
    done
    echo "EOF"
  } >>"$GITHUB_OUTPUT"
fi

exit "$failed"
