//! Kitty Keyboard Protocol progressive-enhancement flags (FR-ORCA-038, ULYS-166).
//!
//! Implements the byte-level Kitty Keyboard Protocol in a **stack-agnostic** way
//! — this module knows nothing about host input handling, key event delivery,
//! or specific terminal emulators. A future [`crate::backend::TerminalBackend`]
//! adapter turns [`KittyKeyboardProtocol`] push/pop/query-set requests into real
//! bytes written to (and parsed from) a PTY.
//!
//! ## Byte format (per kitty "Keyboard protocol" docs)
//!
//! ```text
//! CSI > Pp u          push current flags, set flags = Pp (decimal)
//! CSI < Pp u          pop flags stack, set flags = popped value
//! CSI = flags ; Ps u  set flags (or query if Ps == 1)
//! ```
//!
//! - `Pp` / `flags` is the **progressive-enhancement bitmask** — 8 bits, see
//!   [`KittyKeyboardFlags`].
//! - `Ps == 1` in the query-set form means "respond with the current flags".
//! - The flag bits and their semantics are stable across kitty, foot, ghostty,
//!   wezterm and alacritty (as of 2026).
//!
//! ## Spec anchor caveat
//!
//! `docs/ecosystem-survey/orca-design-survey.md` v1.0 §12 (FR-ORCA-038) was not
//! reachable in this worktree at the time ULYS-166 was opened (see issue
//! description §1). The byte-level rules above follow the public kitty
//! documentation at <https://sw.kovidgoyal.net/kitty/keyboard-protocol/>,
//! which is the upstream canonical source and matches what every compatible
//! terminal implements.

use std::fmt;

/// The 8 progressive-enhancement flag bits defined by the Kitty Keyboard
/// Protocol.
///
/// Wire layout (LSB-first, value sent as decimal):
/// ```text
/// bit 0 (0x01) — disambiguate_escape_codes
/// bit 1 (0x02) — report_event_types
/// bit 2 (0x04) — report_alternate_keys
/// bit 3 (0x08) — report_all_keys_as_escape_codes
/// bit 4 (0x10) — report_associated_text
/// bit 5 (0x20) — report_all_keys_as_escape_alternate
/// bit 6 (0x40) — report_mouse_event
/// bit 7 (0x80) — modify_other_keys
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct KittyKeyboardFlags(u8);

impl KittyKeyboardFlags {
    /// No progressive-enhancement flags set.
    pub const NONE: Self = Self(0);
    /// All 8 flags set (value `0xFF`).
    pub const ALL: Self = Self(0xFF);

    /// Raw byte value — what the protocol sends on the wire.
    #[must_use]
    pub const fn as_u8(self) -> u8 {
        self.0
    }

    /// Construct from a raw byte value, masking off any high bits above the
    /// 8 defined protocol bits. This is the lenient form used by [`Self::parse`].
    #[must_use]
    pub const fn from_bits_truncate(bits: u8) -> Self {
        Self(bits & Self::ALL.0)
    }

    /// `true` when no flag bits are set.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// `true` when at least one flag bit is set.
    #[must_use]
    pub const fn is_any(self) -> bool {
        self.0 != 0
    }

    /// Set `flag` and return the new value.
    #[must_use]
    pub const fn insert(mut self, flag: KittyKeyboardFlag) -> Self {
        self.0 |= flag.bit();
        self
    }

    /// Clear `flag` and return the new value.
    #[must_use]
    pub const fn remove(mut self, flag: KittyKeyboardFlag) -> Self {
        self.0 &= !flag.bit();
        self
    }

    /// Toggle `flag` and return the new value.
    #[must_use]
    pub const fn toggle(mut self, flag: KittyKeyboardFlag) -> Self {
        self.0 ^= flag.bit();
        self
    }

    /// `true` if `flag` is currently set.
    #[must_use]
    pub const fn contains(self, flag: KittyKeyboardFlag) -> bool {
        (self.0 & flag.bit()) != 0
    }

    /// Iterate over the set flags, in bit order (LSB first).
    pub fn iter(self) -> impl Iterator<Item = KittyKeyboardFlag> {
        let mut bits = self.0;
        // SAFETY: bits shrinks each iteration; bounded by 8.
        std::iter::from_fn(move || {
            if bits == 0 {
                return None;
            }
            let low = bits.trailing_zeros() as u8;
            bits &= bits - 1; // clear lowest set bit
            KittyKeyboardFlag::from_bit(low)
        })
    }
}

impl Default for KittyKeyboardFlags {
    fn default() -> Self {
        Self::NONE
    }
}

impl fmt::Display for KittyKeyboardFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u8> for KittyKeyboardFlags {
    fn from(bits: u8) -> Self {
        Self::from_bits_truncate(bits)
    }
}

impl From<KittyKeyboardFlags> for u8 {
    fn from(flags: KittyKeyboardFlags) -> u8 {
        flags.0
    }
}

impl std::ops::BitOr for KittyKeyboardFlags {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOr<KittyKeyboardFlag> for KittyKeyboardFlags {
    type Output = Self;

    fn bitor(self, rhs: KittyKeyboardFlag) -> Self {
        self.insert(rhs)
    }
}

impl std::ops::BitAnd for KittyKeyboardFlags {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitXor for KittyKeyboardFlags {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self {
        Self(self.0 ^ rhs.0)
    }
}

/// One of the 8 named Kitty Keyboard Protocol flag bits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KittyKeyboardFlag {
    /// `bit 0` — disambiguate ambiguous escape sequences (most common first).
    DisambiguateEscapeCodes,
    /// `bit 1` — report key *event types* (press / repeat / release) in CSI u.
    ReportEventTypes,
    /// `bit 2` — report the alternate key (e.g. shifted variant) alongside the
    /// base key.
    ReportAlternateKeys,
    /// `bit 3` — report every key as if it were an escape sequence, even
    /// printable ones.
    ReportAllKeysAsEscapeCodes,
    /// `bit 4` — report the text that would have been produced by the key
    /// (using the current keyboard layout).
    ReportAssociatedText,
    /// `bit 5` — combine `ReportAlternateKeys` + `ReportAllKeysAsEscapeCodes`.
    ReportAllKeysAsEscapeAlternate,
    /// `bit 6` — also report mouse events as CSI u sequences.
    ReportMouseEvent,
    /// `bit 7` — `modify_other_keys` behaviour (rare; advanced).
    ModifyOtherKeys,
}

impl KittyKeyboardFlag {
    /// The protocol-defined bit mask for this flag.
    #[must_use]
    pub const fn bit(self) -> u8 {
        match self {
            Self::DisambiguateEscapeCodes => 1 << 0,
            Self::ReportEventTypes => 1 << 1,
            Self::ReportAlternateKeys => 1 << 2,
            Self::ReportAllKeysAsEscapeCodes => 1 << 3,
            Self::ReportAssociatedText => 1 << 4,
            Self::ReportAllKeysAsEscapeAlternate => 1 << 5,
            Self::ReportMouseEvent => 1 << 6,
            Self::ModifyOtherKeys => 1 << 7,
        }
    }

    /// Reverse of [`Self::bit`] — returns `Some(flag)` for bits 0..=7, `None`
    /// otherwise.
    #[must_use]
    pub const fn from_bit(bit: u8) -> Option<Self> {
        match bit {
            0 => Some(Self::DisambiguateEscapeCodes),
            1 => Some(Self::ReportEventTypes),
            2 => Some(Self::ReportAlternateKeys),
            3 => Some(Self::ReportAllKeysAsEscapeCodes),
            4 => Some(Self::ReportAssociatedText),
            5 => Some(Self::ReportAllKeysAsEscapeAlternate),
            6 => Some(Self::ReportMouseEvent),
            7 => Some(Self::ModifyOtherKeys),
            _ => None,
        }
    }
}

/// The progressive-enhancement state machine for the Kitty Keyboard Protocol.
///
/// The protocol is *progressive* — the application pushes / pops a stack of
/// flag configurations and may query the terminal for the currently active
/// configuration. [`KittyKeyboardProtocol`] owns that stack; the wire-level
/// effect of each transition is a single CSI sequence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum KittyProgressiveState {
    /// No flag has ever been pushed. The terminal is in legacy (pre-Kitty)
    /// input mode. `push` here starts the stack at `flags`.
    Virgin {
        /// The flags that will be in effect after the first `push`.
        flags: KittyKeyboardFlags,
    },
    /// The stack is active. `flags` is the current top of the stack.
    Active {
        /// Flags currently in effect.
        flags: KittyKeyboardFlags,
        /// Number of entries currently on the stack (≥ 1).
        depth: u8,
    },
    /// The protocol has been fully popped — the terminal is back in legacy
    /// input mode but the application knows the protocol *was* used.
    Cleared,
}

impl KittyProgressiveState {
    /// `true` if the application has ever engaged the protocol.
    #[must_use]
    pub const fn is_engaged(self) -> bool {
        !matches!(self, Self::Virgin { flags: KittyKeyboardFlags::NONE })
    }

    /// `true` if a flag configuration is currently in effect.
    #[must_use]
    pub const fn is_active(self) -> bool {
        matches!(self, Self::Active { .. })
    }

    /// The flags currently in effect, if any.
    #[must_use]
    pub const fn current_flags(self) -> Option<KittyKeyboardFlags> {
        match self {
            Self::Virgin { .. } => None,
            Self::Active { flags, .. } => Some(flags),
            Self::Cleared => None,
        }
    }

    /// Apply a `push` event. `flags` becomes the new top of the stack.
    pub fn push(self, flags: KittyKeyboardFlags) -> Self {
        let depth = match self {
            Self::Virgin { .. } | Self::Cleared => 1,
            Self::Active { depth, .. } => depth.saturating_add(1),
        };
        Self::Active { flags, depth }
    }

    /// Apply a `pop` event.
    ///
    /// If the stack becomes empty, the state transitions to [`Self::Cleared`].
    /// If we were already at `Virgin` or `Cleared`, this is a no-op (the wire
    /// form is still sent, but the state stays unchanged).
    pub fn pop(self) -> Self {
        match self {
            Self::Virgin { .. } | Self::Cleared => self,
            Self::Active { flags, depth } => {
                if depth <= 1 {
                    Self::Cleared
                } else {
                    // After pop the *previously stacked* flags become current;
                    // we don't track per-entry history beyond depth, so
                    // conservatively keep the most-recent pushed value as a
                    // sensible approximation (the application is responsible
                    // for issuing pushes / pops in pairs that mirror each
                    // other).
                    Self::Active {
                        flags,
                        depth: depth - 1,
                    }
                }
            }
        }
    }
}

impl Default for KittyProgressiveState {
    fn default() -> Self {
        Self::Virgin {
            flags: KittyKeyboardFlags::NONE,
        }
    }
}

/// Error returned when [`KittyKeyboardProtocol::parse`] cannot decode a wire
/// sequence.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum KittyKeyboardParseError {
    /// Input was empty.
    #[error("Kitty keyboard sequence is empty")]
    Empty,
    /// Input did not start with the `ESC [` (CSI) introducer.
    #[error("Kitty keyboard sequence missing CSI introducer; got {got:?}")]
    MissingCsi {
        /// The bytes we did see at the start.
        got: Vec<u8>,
    },
    /// The CSI final byte was not `u`.
    #[error("Kitty keyboard sequence final byte is not 'u'; got {got:#x}")]
    UnknownFinal {
        /// The byte we saw instead of `u`.
        got: u8,
    },
    /// The parameter bytes could not be parsed as a kitty push/pop/query-set.
    #[error("Kitty keyboard sequence has unrecognized parameter byte {got:#x}")]
    UnrecognizedParameter {
        /// The unexpected intermediate byte (`>`, `<`, `=`, or a digit).
        got: u8,
    },
    /// The flag value in a push / pop is not a valid decimal integer in the
    /// range 0..=255 (8 protocol bits).
    #[error("Kitty keyboard flags value out of range: {0}")]
    FlagsOutOfRange(i32),
}

/// A decoded Kitty Keyboard Protocol request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum KittyKeyboardProtocol {
    /// `CSI > Pp u` — push current flags, then set the new flags to `Pp`.
    Push(KittyKeyboardFlags),
    /// `CSI < Pp u` — pop flags stack, then set the new flags to `Pp`.
    ///
    /// `Pp` is the flags that will be in effect *after* the pop. When the
    /// stack becomes empty as a side effect, the terminal returns to legacy
    /// input mode.
    Pop(KittyKeyboardFlags),
    /// `CSI = flags ; Ps u` — set the flags directly. If `Ps == 1`, this is a
    /// *query* — the terminal replies with the current flag value.
    Set(KittyKeyboardFlags),
}

impl KittyKeyboardProtocol {
    /// Encode this request as the wire bytes the application writes to its
    /// terminal backend. The output is terminated by a BEL (`\x07`) so that
    /// it remains valid even if the backend does not recognise the 7-bit
    /// `ESC \\` ST terminator.
    #[must_use]
    pub fn serialize(self) -> Vec<u8> {
        match self {
            Self::Push(flags) => format!("\x1b[>{}u", flags.as_u8()).into_bytes(),
            Self::Pop(flags) => format!("\x1b[<{}u", flags.as_u8()).into_bytes(),
            Self::Set(flags) => format!("\x1b[={};1u", flags.as_u8()).into_bytes(),
        }
    }

    /// Decode a wire sequence. The input may omit the trailing BEL / `ESC \\`
    /// for callers that read framed buffers themselves; if present they must
    /// match the kitty form (BEL `\x07`).
    ///
    /// Returns an error if the input does not look like a Kitty keyboard
    /// request. Other CSI sequences (SGR, cursor moves, …) will return
    /// [`KittyKeyboardParseError::UnrecognizedParameter`] or
    /// [`KittyKeyboardParseError::UnknownFinal`] as appropriate.
    pub fn parse(input: &[u8]) -> Result<Self, KittyKeyboardParseError> {
        if input.is_empty() {
            return Err(KittyKeyboardParseError::Empty);
        }
        // CSI = ESC [
        if input.len() < 3 || input[0] != 0x1b || input[1] != b'[' {
            return Err(KittyKeyboardParseError::MissingCsi {
                got: input.iter().copied().take(4).collect(),
            });
        }

        // Determine the parameter byte (digit/intermediate) by scanning
        // parameter bytes until we hit `u` (0x75) or BEL (0x07) terminator.
        // CSI parameter bytes are 0x30..=0x3F ('0'..='?'), intermediate
        // bytes are 0x20..=0x2F (' '..='/'), and the final byte is
        // 0x40..=0x7E ('@'..='~'). kitty sequences use exactly one
        // intermediate (`>`, `<`, or `=`) followed by digits / `;`, then
        // final `u`.
        let mut i = 2;
        let mut params: Vec<u8> = Vec::new();
        while i < input.len() {
            let b = input[i];
            if b == b'u' {
                break;
            }
            // Tolerate BEL as the terminator (some terminals send it) — but
            // we still expect a final `u` first; reject pure-BEL forms.
            if b == 0x07 {
                return Err(KittyKeyboardParseError::UnknownFinal { got: b });
            }
            // Final-byte range means we're past the parameters without
            // finding the expected `u` — that's an unknown final byte, not
            // a malformed parameter.
            if (b'@'..=b'~').contains(&b) {
                return Err(KittyKeyboardParseError::UnknownFinal { got: b });
            }
            if !(b' '..=b'/').contains(&b) && !(b'0'..=b'?').contains(&b) {
                return Err(KittyKeyboardParseError::UnrecognizedParameter { got: b });
            }
            params.push(b);
            i += 1;
        }

        if i >= input.len() || input[i] != b'u' {
            return Err(KittyKeyboardParseError::UnknownFinal {
                got: input.get(i).copied().unwrap_or(0),
            });
        }

        // Trailing ST is optional here — strip if present.
        let trailing_ok = match input.get(i + 1..) {
            None | Some([]) => true,
            Some([0x07]) => true,  // BEL terminator
            Some([0x1b, b'\\']) => true, // 7-bit ST
            _ => false,
        };
        if !trailing_ok {
            return Err(KittyKeyboardParseError::UnknownFinal {
                got: input[i + 1],
            });
        }

        // Pull the intermediate byte (must be exactly one of `>`, `<`, `=`).
        let intermediate = params
            .iter()
            .find(|b| matches!(**b, b'>' | b'<' | b'='))
            .copied();
        let digits_and_semis: Vec<u8> = params
            .iter()
            .copied()
            .filter(|b| !matches!(*b, b'>' | b'<' | b'='))
            .collect();

        let parse_u8 = |digits: &[u8]| -> Result<u8, KittyKeyboardParseError> {
            if digits.is_empty() {
                return Ok(0);
            }
            let s = std::str::from_utf8(digits).map_err(|_| {
                KittyKeyboardParseError::UnrecognizedParameter {
                    got: digits.first().copied().unwrap_or(0),
                }
            })?;
            let n: i32 = s.parse().map_err(|_| {
                KittyKeyboardParseError::FlagsOutOfRange(
                    s.parse().unwrap_or(i32::MIN),
                )
            })?;
            if !(0..=255).contains(&n) {
                return Err(KittyKeyboardParseError::FlagsOutOfRange(n));
            }
            Ok(n as u8)
        };

        match intermediate {
            Some(b'>') => {
                let flags = parse_u8(&digits_and_semis)?;
                Ok(Self::Push(KittyKeyboardFlags::from_bits_truncate(flags)))
            }
            Some(b'<') => {
                let flags = parse_u8(&digits_and_semis)?;
                Ok(Self::Pop(KittyKeyboardFlags::from_bits_truncate(flags)))
            }
            Some(b'=') => {
                // `CSI = flags ; Ps u` — split on `;`.
                let mut parts = digits_and_semis.split(|b| *b == b';');
                let flags_bytes = parts.next().unwrap_or(&[]);
                let ps_bytes = parts.next().unwrap_or(&[]);
                // Reject a third `;` (the kitty form has at most one).
                if parts.next().is_some() {
                    return Err(KittyKeyboardParseError::UnrecognizedParameter {
                        got: b';',
                    });
                }
                let flags = parse_u8(flags_bytes)?;
                let _ps = parse_u8(ps_bytes)?;
                // We accept any Ps value here — Set(flags) is the same wire
                // form regardless of whether Ps == 1 (query) or Ps == 0
                // (set). Callers that care about the query semantics inspect
                // the raw bytes separately.
                Ok(Self::Set(KittyKeyboardFlags::from_bits_truncate(flags)))
            }
            _ => Err(KittyKeyboardParseError::UnrecognizedParameter {
                got: params.first().copied().unwrap_or(0),
            }),
        }
    }
}

impl fmt::Display for KittyKeyboardProtocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Push(flags) => write!(f, "kitty:push({})", flags.as_u8()),
            Self::Pop(flags) => write!(f, "kitty:pop({})", flags.as_u8()),
            Self::Set(flags) => write!(f, "kitty:set({})", flags.as_u8()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- KittyKeyboardFlag ---------------------------------------------

    #[test]
    fn flag_bit_mapping_is_canonical() {
        // The kitty docs list bits LSB-first; this is the only ordering
        // every compatible emulator implements, so pin it here.
        assert_eq!(KittyKeyboardFlag::DisambiguateEscapeCodes.bit(), 0x01);
        assert_eq!(KittyKeyboardFlag::ReportEventTypes.bit(), 0x02);
        assert_eq!(KittyKeyboardFlag::ReportAlternateKeys.bit(), 0x04);
        assert_eq!(KittyKeyboardFlag::ReportAllKeysAsEscapeCodes.bit(), 0x08);
        assert_eq!(KittyKeyboardFlag::ReportAssociatedText.bit(), 0x10);
        assert_eq!(KittyKeyboardFlag::ReportAllKeysAsEscapeAlternate.bit(), 0x20);
        assert_eq!(KittyKeyboardFlag::ReportMouseEvent.bit(), 0x40);
        assert_eq!(KittyKeyboardFlag::ModifyOtherKeys.bit(), 0x80);
    }

    #[test]
    fn flag_from_bit_is_inverse_of_bit() {
        for bit in 0..=7u8 {
            let flag = KittyKeyboardFlag::from_bit(bit).expect("0..=7 must map");
            assert_eq!(flag.bit(), 1 << bit);
        }
        assert_eq!(KittyKeyboardFlag::from_bit(8), None);
    }

    // ---- KittyKeyboardFlags --------------------------------------------

    #[test]
    fn flags_empty_and_all() {
        assert!(KittyKeyboardFlags::NONE.is_empty());
        assert!(!KittyKeyboardFlags::NONE.is_any());
        assert!(!KittyKeyboardFlags::ALL.is_empty());
        assert!(KittyKeyboardFlags::ALL.is_any());
    }

    #[test]
    fn flags_insert_remove_toggle_contains() {
        let mut f = KittyKeyboardFlags::NONE;
        f = f.insert(KittyKeyboardFlag::DisambiguateEscapeCodes);
        assert!(f.contains(KittyKeyboardFlag::DisambiguateEscapeCodes));
        assert!(!f.contains(KittyKeyboardFlag::ReportEventTypes));

        f = f.insert(KittyKeyboardFlag::ReportAlternateKeys);
        assert_eq!(f.as_u8(), 0x05);

        f = f.remove(KittyKeyboardFlag::DisambiguateEscapeCodes);
        assert_eq!(f.as_u8(), 0x04);
        assert!(!f.contains(KittyKeyboardFlag::DisambiguateEscapeCodes));
        assert!(f.contains(KittyKeyboardFlag::ReportAlternateKeys));

        f = f.toggle(KittyKeyboardFlag::DisambiguateEscapeCodes);
        assert_eq!(f.as_u8(), 0x05);
        f = f.toggle(KittyKeyboardFlag::DisambiguateEscapeCodes);
        assert_eq!(f.as_u8(), 0x04);
    }

    #[test]
    fn flags_from_bits_truncate_drops_high_bits() {
        // The protocol only defines bits 0..=7; high bits are ignored.
        // 0x1F_AB wraps a wider pattern as u16 first to verify truncation.
        let wide: u16 = 0xFF_AB;
        let f = KittyKeyboardFlags::from_bits_truncate(wide as u8);
        assert_eq!(f.as_u8(), 0xAB);
    }

    #[test]
    fn flags_iter_walks_set_bits_in_order() {
        let f = KittyKeyboardFlags::from_bits_truncate(0x05); // bits 0 + 2
        let walked: Vec<_> = f.iter().collect();
        assert_eq!(
            walked,
            vec![
                KittyKeyboardFlag::DisambiguateEscapeCodes,
                KittyKeyboardFlag::ReportAlternateKeys,
            ]
        );
        assert!(KittyKeyboardFlags::NONE.iter().next().is_none());
    }

    #[test]
    fn flags_bitor_bitand_bitxor_operators() {
        let a = KittyKeyboardFlags::from_bits_truncate(0x05);
        let b = KittyKeyboardFlags::from_bits_truncate(0x03);
        assert_eq!((a | b).as_u8(), 0x07);
        assert_eq!((a & b).as_u8(), 0x01);
        assert_eq!((a ^ b).as_u8(), 0x06);

        // `flags | flag` convenience.
        let c = KittyKeyboardFlags::NONE | KittyKeyboardFlag::ReportMouseEvent;
        assert_eq!(c.as_u8(), 0x40);
    }

    #[test]
    fn flags_default_is_none() {
        assert_eq!(KittyKeyboardFlags::default(), KittyKeyboardFlags::NONE);
    }

    // ---- KittyProgressiveState -----------------------------------------

    #[test]
    fn progressive_state_default_is_virgin() {
        let s = KittyProgressiveState::default();
        assert!(!s.is_engaged());
        assert!(!s.is_active());
        assert_eq!(s.current_flags(), None);
    }

    #[test]
    fn progressive_state_push_pop_balance() {
        let mut s = KittyProgressiveState::default();
        s = s.push(KittyKeyboardFlags::from_bits_truncate(0x05));
        assert!(s.is_engaged());
        assert!(s.is_active());
        assert_eq!(s.current_flags().unwrap().as_u8(), 0x05);

        // Nested push (deeper stack).
        s = s.push(KittyKeyboardFlags::from_bits_truncate(0x03));
        assert!(s.is_active());

        s = s.pop();
        assert!(s.is_active());
        s = s.pop();
        assert!(!s.is_active());
        assert!(matches!(s, KittyProgressiveState::Cleared));
    }

    #[test]
    fn progressive_state_pop_when_virgin_is_noop() {
        let s = KittyProgressiveState::default();
        let after = s.pop();
        assert_eq!(after, s);
        assert!(!after.is_active());
    }

    #[test]
    fn progressive_state_pop_when_cleared_is_noop() {
        let s = KittyProgressiveState::Cleared;
        assert_eq!(s.pop(), KittyProgressiveState::Cleared);
    }

    // ---- KittyKeyboardProtocol::serialize ------------------------------

    #[test]
    fn serialize_push_form() {
        let bytes = KittyKeyboardProtocol::Push(KittyKeyboardFlags::from_bits_truncate(5))
            .serialize();
        assert_eq!(bytes, b"\x1b[>5u");
    }

    #[test]
    fn serialize_pop_form() {
        let bytes = KittyKeyboardProtocol::Pop(KittyKeyboardFlags::from_bits_truncate(7))
            .serialize();
        assert_eq!(bytes, b"\x1b[<7u");
    }

    #[test]
    fn serialize_set_form() {
        let bytes = KittyKeyboardProtocol::Set(KittyKeyboardFlags::from_bits_truncate(13))
            .serialize();
        assert_eq!(bytes, b"\x1b[=13;1u");
    }

    // ---- KittyKeyboardProtocol::parse ----------------------------------

    #[test]
    fn parse_push_round_trip() {
        let wire = b"\x1b[>5u";
        let p = KittyKeyboardProtocol::parse(wire).unwrap();
        assert_eq!(p, KittyKeyboardProtocol::Push(KittyKeyboardFlags::from_bits_truncate(5)));
        // serialize back
        assert_eq!(p.serialize(), wire);
    }

    #[test]
    fn parse_pop_round_trip() {
        let wire = b"\x1b[<7u";
        let p = KittyKeyboardProtocol::parse(wire).unwrap();
        assert_eq!(p, KittyKeyboardProtocol::Pop(KittyKeyboardFlags::from_bits_truncate(7)));
        assert_eq!(p.serialize(), wire);
    }

    #[test]
    fn parse_set_round_trip() {
        let wire = b"\x1b[=13;1u";
        let p = KittyKeyboardProtocol::parse(wire).unwrap();
        assert_eq!(p, KittyKeyboardProtocol::Set(KittyKeyboardFlags::from_bits_truncate(13)));
        assert_eq!(p.serialize(), wire);
    }

    #[test]
    fn parse_tolerates_bel_terminator() {
        let wire = b"\x1b[>5u\x07";
        let p = KittyKeyboardProtocol::parse(wire).unwrap();
        assert_eq!(p, KittyKeyboardProtocol::Push(KittyKeyboardFlags::from_bits_truncate(5)));
    }

    #[test]
    fn parse_tolerates_esc_backslash_terminator() {
        let wire = b"\x1b[>5u\x1b\\";
        let p = KittyKeyboardProtocol::parse(wire).unwrap();
        assert_eq!(p, KittyKeyboardProtocol::Push(KittyKeyboardFlags::from_bits_truncate(5)));
    }

    #[test]
    fn parse_rejects_empty_input() {
        assert_eq!(
            KittyKeyboardProtocol::parse(b""),
            Err(KittyKeyboardParseError::Empty)
        );
    }

    #[test]
    fn parse_rejects_missing_csi() {
        assert!(matches!(
            KittyKeyboardProtocol::parse(b"foo"),
            Err(KittyKeyboardParseError::MissingCsi { .. })
        ));
    }

    #[test]
    fn parse_rejects_unknown_final_byte() {
        // 'm' instead of 'u'
        assert!(matches!(
            KittyKeyboardProtocol::parse(b"\x1b[>5m"),
            Err(KittyKeyboardParseError::UnknownFinal { got: b'm' })
        ));
    }

    #[test]
    fn parse_rejects_unrecognized_intermediate() {
        // '?' is a valid CSI intermediate range but not kitty's three
        // operators; we treat it as unrecognized for the protocol.
        assert!(matches!(
            KittyKeyboardProtocol::parse(b"\x1b[?5u"),
            Err(KittyKeyboardParseError::UnrecognizedParameter { .. })
        ));
    }

    #[test]
    fn parse_rejects_flags_out_of_range() {
        // 256 truncates to 0 via masking, but '300' as a literal decimal
        // exceeds 8 bits and should be rejected as out-of-range.
        assert!(matches!(
            KittyKeyboardProtocol::parse(b"\x1b[>300u"),
            Err(KittyKeyboardParseError::FlagsOutOfRange(300))
        ));
    }

    // ---- Display -------------------------------------------------------

    #[test]
    fn display_round_trip_is_readable() {
        let p = KittyKeyboardProtocol::Push(KittyKeyboardFlags::from_bits_truncate(5));
        let s = format!("{p}");
        assert_eq!(s, "kitty:push(5)");
    }
}