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
  or plugin ABI. In the current integration, however, adapters are compiled
  into `nema-codec` and its WASM binding. Independently installing and running
  a third-party interpreter without shipping a new application artifact is not
  supported yet; the isolated plugin runtime below is still a known gap.
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
configured shell command in the Star process. The product does not yet have
the isolated NEMA plugin request/response runtime, so third-party plugin
execution and independent plugin installation are not available.

Reusable infrastructure found: `domain-agent::PluginLifecycleManager` owns
plugin id/version/digest, activation, admission, drain, and removal metadata;
`sandboxd` provides a separate process isolation service with resource,
network, filesystem, and capability policies. Neither currently defines a
fixed NEMA interpreter request/response protocol or a digest-verified worker
launcher, so wiring them together remains a separate runtime gap. The NEMA
registry is ready to dispatch through a trusted adapter for such a host, but
third-party plugin execution is not claimed as available. The legacy adaptive
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
