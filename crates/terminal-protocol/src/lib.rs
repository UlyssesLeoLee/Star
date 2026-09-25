//! `terminal-protocol` — terminal escape-sequence protocol layer for ULYS-104.11 (ULYS-166).
//!
//! This crate is the **protocol-only** layer for two terminal features called out
//! in `docs/ecosystem-survey/orca-design-survey.md` §12 (FR-ORCA-037 / FR-ORCA-038):
//!
//! - [`osc52`] — OSC 52 clipboard read / write / clear, base64-encoded payloads.
//! - [`kitty_kbd`] — Kitty Keyboard Protocol progressive-enhancement flags.
//!
//! The concrete terminal stack (Ghostty / xterm.js / alacritty / …) is **not** chosen
//! yet — that decision lives on ULYS-164 (ULYS-104.10). ULYS-161 (ULYS-104.6) is the
//! shared OSC parser infrastructure. This crate therefore:
//!
//! 1. Owns the *data shapes* and *byte-level serialize/parse* (so a future backend
//!    can just feed bytes in / take bytes out).
//! 2. Exposes a [`backend::TerminalBackend`] trait that the future stack-specific
//!    adapter will implement — so the protocol stays stack-agnostic.
//!
//! It does **not** touch host OS clipboards, PTYs, panes, or rendering. Those
//! responsibilities arrive when ULYS-164 / ULYS-161 land and a backend plugs in.
//!
//! ## Spec anchors
//!
//! Spec source `docs/ecosystem-survey/orca-design-survey.md` (v1.0, FR-ORCA-037/038)
//! was not reachable in this worktree at issue-create time (see ULYS-166 description
//! §1 / §3). The byte-level format implemented here follows the public, widely
//! cited references so the crate stays correct without the local anchor:
//!
//! - OSC 52: `man 1 xterm` OSC 52 ; base64 ; ESC `\\` (CSI `Ps ; Pd` Pt ST).
//!   Payload selector `c` = clipboard; `p` = primary selection (X11);
//!   `s` = secondary selection (X11); `?` = read request.
//! - Kitty Keyboard Protocol: kitty docs "Kitty keyboard protocol" page,
//!   `CSI > Pp u` push / `CSI < Pp u` pop / `CSI = flags ; Ps u` query-set,
//!   8 flag bits (`disambiguate_escape_codes`, `report_event_types`,
//!   `report_alternate_keys`, `report_all_keys_as_escape_codes`,
//!   `report_associated_text`, `report_all_keys_as_escape_alternate`,
//!   `report_mouse_event`, `modify_other_keys`).
//!
//! ## Workspace lint posture
//!
//! `missing_docs = deny` and `unreachable_pub = deny` are workspace-wide. Every
//! public item below has a doc comment.

#![deny(missing_docs)]
#![deny(unreachable_pub)]

pub mod backend;
pub mod kitty_kbd;
pub mod osc52;

// Re-export the most useful top-level types so downstream crates can write
// `terminal_protocol::{Osc52Clipboard, KittyKeyboardFlags, TerminalBackend}`
// without redundant module paths.
pub use backend::{MockBackend, TerminalBackend};
pub use kitty_kbd::{KittyKeyboardFlags, KittyKeyboardProtocol, KittyProgressiveState};
pub use osc52::{Osc52Clipboard, Osc52Error, Osc52Operation, Osc52ParseError, Osc52Selector};
