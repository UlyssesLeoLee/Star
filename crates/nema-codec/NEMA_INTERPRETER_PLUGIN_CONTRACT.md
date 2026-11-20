# NEMA interpreter plugin contract

The Rust `nema-codec` crate owns the lossless document model and dispatches
through an exact interpreter profile. Grammar revisions and interpreter
implementation versions are separate. Every profile pins the interpreter ABI,
AST schema, grammar source digest, and Rust parser source artifact digest.

## Compatibility rules

- A `.nema` document is parsed only with the profile selected by its workspace
  binding or by the import operation. The current text syntax has no reliable
  grammar-version marker, so the registry never guesses or chooses “latest”.
- An unknown profile, duplicate profile, unsupported ABI/schema, or missing
  artifact fails closed. Existing documents keep their pinned interpreter
  profile until an explicit migration is requested.
- The stable Rust adapter contract allows a grammar change that maps to the
  existing AST schema to be implemented without changing the document model
  or plugin ABI. Adapters are compiled into `nema-codec` and its WASM binding,
  or installed behind the isolated plugin runtime (see "Execution and
  lifecycle"): the fixed protocol, digest-verified artifacts, resource
  limits and independent installation are implemented as of the P0 .nema
  lane; the sandboxd glue seam and product wiring are tracked in
  `docs/nema/KNOWN-GAPS.md`.
- The captured 2026-11-15 matrix changes continuation syntax to an explicit
  column index. The crate keeps the earlier indentation-based v1 parser and
  adds an independent indexed-continuation v2 parser; a profile selects which
  behavior applies to a file.
- Formatting is canonical for the selected profile. The registry parses the
  formatted result with that same profile and rejects output that does not
  reconstruct the input AST, including Value/Null/Empty/Missing distinctions.
- Newly created task sidecars and newly imported files in the NEMA page select
  an installed profile, format and parse through Rust, and persist the complete
  profile with the task entry. File watcher updates and opening a task reuse
  that exact profile. Existing persisted entries without a profile remain on
  the explicitly legacy task projection; they are not silently upgraded.
- The browser bridge serializes Rust `u64` domain IDs and revisions as decimal
  strings, so JavaScript cannot round them above `Number.MAX_SAFE_INTEGER`.
- Each plugin release carries a fixture snapshot with exact source hashes.
  Prior snapshots remain in the compatibility suite; proposal examples are
  tracked separately and are not treated as accepted grammar.

## Execution and lifecycle

`NemaInterpreterAdapter` is a trusted host-side extension seam. The bundled
v1/v2 interpreters are native Rust and are the only adapters currently
executable through the product path. A third-party interpreter must run behind
an isolated worker or sandboxed WASM host with bounded input, output, memory,
CPU/time, and capabilities. Do not `dlopen` a plugin DLL or run an arbitrary
configured shell command in the Star process.

As of the P0 .nema lane (2026-11-20), the isolated plugin runtime exists as
`nema-codec::plugin_runtime` and the sandboxd-side worker host as
`sandboxd::nema_worker`, with these fixed contracts:

- **Protocol**: `NEMA_PLUGIN_PROTOCOL_VERSION = 1`. Requests are serde JSON
  (`ParseRequest` / `FormatRequest` with the exact `InterpreterProfile`,
  source or tagged AST, and `ResourceLimits`); responses carry exactly one of
  `ast`/`text` or a `PluginErrorPayload` (`resource_limit` / `timeout` /
  `out_of_memory` map to `InterpreterError::ResourceLimit`). A version
  mismatch or malformed response is rejected as `ProtocolViolation`.
- **Artifacts**: `PluginArtifact { sha256, payload }` is verified before any
  record or adapter exists; a mismatch fails with
  `InterpreterError::DigestMismatch` carrying both digests. Digest computation
  is a dependency-free SHA-256 (`nema-codec::sha256`), used for integrity
  verification only — not a MAC or signature.
- **Limits**: input/output bytes, domains, records, columns, total cells are
  enforced host-side; memory/CPU/time are reported by the worker host and
  mapped to `InterpreterError::ResourceLimit`. Oversized input remains
  `InputTooLarge`; oversized output `OutputTooLarge`/`ResourceLimit`.
- **Worker host (path a — sandboxd)**: `sandboxd::nema_worker` verifies the
  worker executable digest again at launch, freezes a restricted 4-dimension
  `SessionSpec` (network BLOCK_ALL, capability DropAll, fs allowlist = work
  dir only), executes `argv = [program] + args + [request-file, response-file]`
  by direct exec (never a shell), and enforces a response byte cap. The
  `nema-worker-fixture` mode of the sandboxd binary is the reference worker
  for the protocol contract. The cross-crate glue
  (`SandboxdNemaPluginHost: NemaPluginHost` in nema-codec) is blocked by the
  frozen `Cargo.lock` and remains a known gap; nema-codec tests use an
  in-memory protocol host to exercise the seam. Path b (in-process WASM
  host) is not implemented; the `NemaPluginHost` trait is the seam for it.
- **Independent installation**: `NemaPluginInstaller` ports the
  `domain-agent::PluginLifecycleManager` state machine
  (Installed → Active → Draining → Drained → Uninstalled): digest + ABI/AST
  checks at install, duplicate `plugin_id@version` rejected, upgrade installs
  the new version and drains the old one, uninstall removes the adapter so
  the profile becomes unregistered again. No built-in fallback anywhere.
- **File binding and migration**: every file pins a `NemaFileBinding`
  (exact profile + source digest). Old files keep their bound interpreter;
  `preview_migration` never mutates anything and
  `execute_migration` returns the original source for an explicit backup.
  A migrated binding's digest is over the canonical target text.

Known gaps after this lane (see `docs/nema/KNOWN-GAPS.md`): the
nema-codec ↔ sandboxd glue dependency (Cargo.lock frozen), Windows v0.1
policy-enforcement levels for network/fs/capability dims, the
`wasm32-unknown-unknown` build verification, and persistence wiring. Claims
that third-party plugin execution is available in the product must not be
made until the glue and persistence exist with test evidence.

Reusable infrastructure found: `domain-agent::PluginLifecycleManager` owns
plugin id/version/digest, activation, admission, drain, and removal metadata
(its state machine is ported 1:1 into `NemaPluginInstaller`);
`sandboxd` provides a separate process isolation service with resource,
network, filesystem, and capability policies (`sandboxd::nema_worker` builds
the NEMA-specific restricted session on top). The legacy adaptive
`nema-codec::interpreter` probe is not used for profile dispatch because it can
invoke Python/CLI and silently fall back to the built-in parser.

## Updating grammar snapshots

1. Capture the grammar page, matrix, and reference parser as immutable inputs;
   record each SHA-256 and the source repository revision/dirty state.
2. Treat a live-site update as a new snapshot. Do not overwrite old fixture
   outcomes or mix a newer matrix with an older grammar page without recording
   that mismatch. The local 2026-11-15 matrix is pinned at
   `ec274eda324c8fc88edc88a4bbc5afe3b508d4b4ca953ead173809aca6ae058a`; its
   local reference parser is pinned at
   `d2f2d2fead79c4f3127a695e8ef5906880c42f5811e7c14c4be5aed5cd851fe8`.
   The local `grammar.html` snapshot is pinned at
   `67110ac289dfd520c43e6c6055c042f9d5a0c18049471d066395a92177f94cde`.
   The bundled Rust parser source artifact is pinned at
   `4ae855e1da4c595d0d146183c304bee966c5fb2f34a846fd0da3d224d7311707`.
3. Build a new interpreter plugin when the grammar behavior changes. Keep
   earlier profiles available and test both old and new fixtures.
4. Run status classification, semantic-value comparison, parse/format/parse
   round trips, canonical-output idempotence, resource-limit checks, and
   unknown-profile rejection.
5. Promote a plugin profile explicitly. Never auto-upgrade files or rewrite
   their canonical text merely because the website changed.

The live web page was not retrievable during this integration. The accepted
snapshot is therefore identified by local source-file hashes and is not
asserted to match the currently deployed page. The local `.NEMA` Git status
could not be read, so repository revision/dirty state remains unknown. The
earlier grammar fixture is preserved independently rather than upgraded in
place. New grammar snapshots can be selected without reinterpreting pinned
entries; no claim is made that a third-party isolated interpreter runtime is
already available.
