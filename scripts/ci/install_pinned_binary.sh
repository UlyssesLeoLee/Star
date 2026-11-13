#!/usr/bin/env bash
# Install a pinned, checksum-verified prebuilt binary from a .tar.gz release asset.
#
# Usage: install_pinned_binary.sh <url> <sha256> <binary-name> [install-dir]
#
# Why a script instead of inline `tar -C /usr/local/bin`:
#   release tarballs ship the binary inside a versioned sub-directory
#   (e.g. cargo-audit-x86_64-unknown-linux-musl-v0.22.2/cargo-audit). Extracting
#   straight into the install dir leaves it at a nested path and a later
#   `chmod +x /usr/local/bin/<name>` fails (license-gate run 43329468948).
#   This script extracts to a temp dir, locates the binary by name, and
#   installs it, so a layout change fails loudly instead of silently.
#
# The expected SHA-256 must come from the upstream release API `digest` field
# (gh api repos/<owner>/<repo>/releases/tags/<tag> --jq '.assets[]|{name,digest}')
# and is verified before anything is extracted. Never read it from the
# downloaded asset itself.
# @cypher schema=1
# MERGE (self:File {path:"scripts/ci/install_pinned_binary.sh"})
# MERGE (args:Config {id:"scripts/ci/install_pinned_binary.sh::args",name:"url sha256 binary-name [install-dir]"})
# MERGE (tmp:Config {id:"scripts/ci/install_pinned_binary.sh::tmp",name:"mktemp -d, removed by EXIT trap"})
# MERGE (curl:ExternalService {id:"curl"})
# MERGE (sha256sum:ExternalService {id:"sha256sum"})
# MERGE (tar:ExternalService {id:"tar"})
# MERGE (install:ExternalService {id:"install"})
# MERGE (self)-[:READS]->(args)
# MERGE (self)-[:WRITES]->(tmp)
# MERGE (self)-[:CALLS]->(curl)
# MERGE (self)-[:CALLS]->(sha256sum)
# MERGE (self)-[:CALLS]->(tar)
# MERGE (self)-[:CALLS]->(install)
# @endcypher
set -euo pipefail

if [ "$#" -lt 3 ] || [ "$#" -gt 4 ]; then
  echo "usage: $0 <url> <sha256> <binary-name> [install-dir]" >&2
  exit 2
fi

url="$1"
expected_sha="$2"
bin_name="$3"
dest_dir="${4:-/usr/local/bin}"

case "$expected_sha" in
  *[!0-9a-f]* | "")
    echo "::error::sha256 must be 64 lowercase hex chars, got: '${expected_sha}'" >&2
    exit 2
    ;;
esac
if [ "${#expected_sha}" -ne 64 ]; then
  echo "::error::sha256 must be 64 lowercase hex chars, got ${#expected_sha}" >&2
  exit 2
fi

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

curl --proto '=https' --tlsv1.2 -fsSL --retry 3 --retry-delay 2 "$url" -o "$tmp/archive"

actual_sha="$(sha256sum "$tmp/archive" | cut -d' ' -f1)"
if [ "$actual_sha" != "$expected_sha" ]; then
  echo "::error::checksum mismatch for $url" >&2
  echo "  expected: $expected_sha" >&2
  echo "  actual:   $actual_sha" >&2
  exit 1
fi

mkdir "$tmp/extract"
tar -xzf "$tmp/archive" -C "$tmp/extract"

src="$(find "$tmp/extract" -type f -name "$bin_name" | head -n 1)"
if [ -z "$src" ]; then
  echo "::error::binary '$bin_name' not found in archive; layout:" >&2
  find "$tmp/extract" -maxdepth 3 | head -n 30 >&2
  exit 1
fi

sudo install -m 0755 "$src" "$dest_dir/$bin_name"
echo "installed $bin_name -> $dest_dir/$bin_name"
"$dest_dir/$bin_name" --version
