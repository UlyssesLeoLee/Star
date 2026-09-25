//! OSC 52 clipboard escape-sequence protocol (FR-ORCA-037, ULYS-166).
//!
//! Implements the OSC 52 (`ESC ] 52 ; <selector> ; <base64> ST`) byte-level
//! protocol in a **stack-agnostic** way — this module knows nothing about
//! host OS clipboards, panes, or PTYs. A future [`crate::backend::TerminalBackend`]
//! adapter turns `Osc52Clipboard` into real side effects.
//!
//! ## Byte format (per `man 1 xterm` OSC 52)
//!
//! ```text
//! ESC ] 5 2 ; <selector> ; <base64-payload-or-empty> ST
//! ```
//!
//! - `<selector>` is one of:
//!   - `c` (default — system clipboard),
//!   - `p` (X11 primary selection),
//!   - `s` (X11 secondary selection),
//!   - `?` (read request — payload empty; the terminal replies by writing
//!     a fresh OSC 52 sequence back).
//! - `<base64-payload-or-empty>` is the clipboard content encoded as
//!   standard base64 (RFC 4648 §4, with `=` padding). Empty = clear.
//! - `ST` is either `ESC \` (the 7-bit terminator) or `BEL` (`\x07`,
//!   the legacy terminator tolerated by xterm).
//!
//! ## Spec anchor caveat
//!
//! `docs/ecosystem-survey/orca-design-survey.md` v1.0 §12 (FR-ORCA-037)
//! was not reachable in this worktree at the time ULYS-166 was opened
//! (see issue description §1). The byte-level rules above follow the
//! public `xterm` man page, which is what every terminal emulator
//! implements for OSC 52 — so the wire format is stable regardless of
//! whether the local anchor comes back later.

use std::fmt;

/// Error returned by [`Osc52Clipboard::serialize`] when an operation cannot
/// be encoded into a wire sequence.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum Osc52Error {
    /// A read request (`?` selector) was passed to `serialize` — read requests
    /// are valid on the wire but must not be confused with `Write`/`Clear`
    /// operations.
    #[error("OSC 52 read requests cannot be serialized as a write/clear operation")]
    ReadRequestNotSerializable,
    /// The payload failed base64 encoding.
    #[error("base64 encoding failed: {0}")]
    Base64Encode(String),
}

/// Error returned by [`Osc52Clipboard::parse`] when an incoming byte sequence
/// cannot be interpreted.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum Osc52ParseError {
    /// Input was empty.
    #[error("OSC 52 sequence is empty")]
    Empty,
    /// Input did not start with the `ESC ]` introducer.
    #[error("OSC 52 sequence missing ESC ] introducer; got {got:?}")]
    MissingIntroducer {
        /// The bytes we did see at the start (for debugging).
        got: Vec<u8>,
    },
    /// The integer after `ESC ]` was not `52`.
    #[error("OSC 52 sequence has unknown command number {0} (expected 52)")]
    UnknownCommand(u16),
    /// Missing the `;` separator between the command and the selector.
    #[error("OSC 52 sequence missing ';' after command")]
    MissingSelectorSeparator,
    /// Missing the `;` separator between the selector and the payload.
    #[error("OSC 52 sequence missing ';' after selector")]
    MissingPayloadSeparator,
    /// The selector character was not one of `c`, `p`, `s`, `?`.
    #[error("OSC 52 sequence has invalid selector byte {0:#x}")]
    InvalidSelector(u8),
    /// The payload is not valid base64 (when non-empty).
    #[error("OSC 52 payload is not valid base64: {0}")]
    InvalidBase64(String),
    /// The sequence did not terminate with `ESC \` or `BEL`.
    #[error("OSC 52 sequence missing ST terminator (ESC \\ or BEL)")]
    MissingTerminator,
}

/// Clipboard selector — which logical clipboard the OSC 52 sequence targets.
///
/// Per `xterm` OSC 52 docs:
/// - `Clipboard` (default) — the system clipboard.
/// - `Primary` / `Secondary` — X11 selections (no-op on non-X11 hosts).
/// - `ReadRequest` — asks the terminal to reply with the current contents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Osc52Selector {
    /// System clipboard (the `c` selector — also the default when omitted).
    Clipboard,
    /// X11 primary selection (the `p` selector).
    Primary,
    /// X11 secondary selection (the `s` selector).
    Secondary,
    /// Read request — payload empty, terminal replies with the contents.
    ReadRequest,
}

impl Osc52Selector {
    /// Single-byte wire encoding.
    #[must_use]
    pub const fn as_byte(self) -> u8 {
        match self {
            Self::Clipboard => b'c',
            Self::Primary => b'p',
            Self::Secondary => b's',
            Self::ReadRequest => b'?',
        }
    }

    /// Parse the single-byte wire encoding. Returns `None` for any byte
    /// outside `{c, p, s, ?}`.
    #[must_use]
    pub const fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            b'c' => Some(Self::Clipboard),
            b'p' => Some(Self::Primary),
            b's' => Some(Self::Secondary),
            b'?' => Some(Self::ReadRequest),
            _ => None,
        }
    }

    /// `true` if this selector is a read request (the `?` form).
    #[must_use]
    pub const fn is_read_request(self) -> bool {
        matches!(self, Self::ReadRequest)
    }
}

impl fmt::Display for Osc52Selector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Clipboard => "c",
            Self::Primary => "p",
            Self::Secondary => "s",
            Self::ReadRequest => "?",
        })
    }
}

/// High-level OSC 52 operation. Distinguishes read requests (no payload) from
/// writes (with payload) and clears (explicit empty payload).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Osc52Operation {
    /// A read request — terminal will reply with the current clipboard
    /// contents on the same selector.
    Read(Osc52Selector),
    /// Write the given bytes to the named clipboard.
    Write {
        /// Which clipboard to write to.
        selector: Osc52Selector,
        /// Raw bytes to encode. UTF-8 text is the common case but the
        /// protocol is byte-transparent.
        data: Vec<u8>,
    },
    /// Clear the named clipboard (equivalent to `Write` with empty data,
    /// but spelled out so callers / logs do not have to special-case it).
    Clear(Osc52Selector),
}

impl Osc52Operation {
    /// Which selector this operation targets.
    #[must_use]
    pub const fn selector(&self) -> Osc52Selector {
        match self {
            Self::Read(s) | Self::Clear(s) => *s,
            Self::Write { selector, .. } => *selector,
        }
    }

    /// `true` if this operation is a read request.
    #[must_use]
    pub const fn is_read_request(&self) -> bool {
        matches!(self, Self::Read(_))
    }
}

/// A parsed OSC 52 sequence — the fully-decoded payload alongside its selector.
///
/// `Osc52Clipboard` is the type most callers will hold. Internally it is the
/// union of [`Osc52Operation`] (logical) and the raw byte form (wire).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Osc52Clipboard {
    /// The logical operation represented by this sequence.
    pub operation: Osc52Operation,
}

impl Osc52Clipboard {
    /// Build a new [`Osc52Clipboard`] for a write to the system clipboard.
    #[must_use]
    pub fn write(data: impl Into<Vec<u8>>) -> Self {
        Self {
            operation: Osc52Operation::Write {
                selector: Osc52Selector::Clipboard,
                data: data.into(),
            },
        }
    }

    /// Build a new [`Osc52Clipboard`] for a write to the named selector.
    #[must_use]
    pub fn write_to(selector: Osc52Selector, data: impl Into<Vec<u8>>) -> Self {
        Self {
            operation: Osc52Operation::Write {
                selector,
                data: data.into(),
            },
        }
    }

    /// Build a read-request [`Osc52Clipboard`] on the system clipboard.
    #[must_use]
    pub fn read() -> Self {
        Self {
            operation: Osc52Operation::Read(Osc52Selector::Clipboard),
        }
    }

    /// Build a read-request [`Osc52Clipboard`] on the named selector.
    #[must_use]
    pub fn read_from(selector: Osc52Selector) -> Self {
        Self {
            operation: Osc52Operation::Read(selector),
        }
    }

    /// Build a clear [`Osc52Clipboard`] on the system clipboard.
    #[must_use]
    pub fn clear() -> Self {
        Self {
            operation: Osc52Operation::Clear(Osc52Selector::Clipboard),
        }
    }

    /// Which selector this sequence targets.
    #[must_use]
    pub const fn selector(&self) -> Osc52Selector {
        self.operation.selector()
    }

    /// `true` if this is a read request.
    #[must_use]
    pub const fn is_read_request(&self) -> bool {
        self.operation.is_read_request()
    }

    /// Encode this operation as an OSC 52 wire sequence (terminated by `ESC \`).
    ///
    /// Read requests cannot be serialized as a "write/clear" operation — they
    /// are still valid on the wire but [`Osc52Clipboard::serialize`] returns
    /// [`Osc52Error::ReadRequestNotSerializable`] if you try.
    pub fn serialize(&self) -> Result<Vec<u8>, Osc52Error> {
        let (selector, payload_b64) = match &self.operation {
            Osc52Operation::Read(_) => return Err(Osc52Error::ReadRequestNotSerializable),
            Osc52Operation::Write { selector, data } => {
                let b64 = base64_encode(data);
                (*selector, b64)
            }
            Osc52Operation::Clear(selector) => (*selector, String::new()),
        };

        // ESC ] 5 2 ; <selector> ; <base64> ESC \
        let mut out = Vec::with_capacity(payload_b64.len() + 12);
        out.extend_from_slice(b"\x1b]52;");
        out.push(selector.as_byte());
        out.push(b';');
        out.extend_from_slice(payload_b64.as_bytes());
        out.extend_from_slice(b"\x1b\\");
        Ok(out)
    }

    /// Parse a wire sequence into an [`Osc52Clipboard`].
    ///
    /// Accepts both terminators: `ESC \` (the 7-bit standard) and `BEL`
    /// (`\x07`, the legacy xterm fallback). Tolerates a `BEL` immediately
    /// after the payload bytes as well as the canonical `ESC \`.
    pub fn parse(input: &[u8]) -> Result<Self, Osc52ParseError> {
        if input.is_empty() {
            return Err(Osc52ParseError::Empty);
        }

        // Strip the introducer. We accept either `ESC ]` (7-bit) or `CSI` as a
        // robustness measure — `CSI 52 ; ... ST` is *not* strictly OSC 52, but
        // a few terminals in the wild emit it. We accept it for the selector
        // and payload sections and only require the OSC-style payload shape.
        let after_intro = if input.starts_with(b"\x1b]") {
            &input[2..]
        } else if input.starts_with(b"\x1b[") {
            // OSC 52 over CSI — treat as if the `]` had been a `[`.
            &input[2..]
        } else {
            return Err(Osc52ParseError::MissingIntroducer {
                got: input.iter().take(4).copied().collect(),
            });
        };

        // Command number — must be `52`. Find the first `;` to terminate it.
        let cmd_end = after_intro
            .iter()
            .position(|&b| b == b';')
            .ok_or(Osc52ParseError::MissingSelectorSeparator)?;
        let cmd_bytes = &after_intro[..cmd_end];
        let cmd: u16 = std::str::from_utf8(cmd_bytes)
            .map_err(|e| Osc52ParseError::InvalidBase64(format!("command not utf-8: {e}")))?
            .parse()
            .map_err(|e: std::num::ParseIntError| {
                Osc52ParseError::InvalidBase64(format!("command not u16: {e}"))
            })?;
        if cmd != 52 {
            return Err(Osc52ParseError::UnknownCommand(cmd));
        }

        // Selector — single byte.
        let after_cmd = &after_intro[cmd_end + 1..];
        if after_cmd.is_empty() {
            return Err(Osc52ParseError::MissingPayloadSeparator);
        }
        let selector_byte = after_cmd[0];
        let selector = Osc52Selector::from_byte(selector_byte)
            .ok_or(Osc52ParseError::InvalidSelector(selector_byte))?;

        // Payload separator. The OSC 52 wire form strictly requires
        // `;<payload>` after the selector, but in practice read requests
        // (`?<ST>`) are commonly sent without the trailing `;` — accept both.
        let after_sel = &after_cmd[1..];
        let payload_with_term = if after_sel.is_empty() {
            return Err(Osc52ParseError::MissingPayloadSeparator);
        } else if after_sel[0] == b';' {
            &after_sel[1..]
        } else if after_sel[0] == b'\x1b' || after_sel[0] == 0x07 {
            // Read-request shortcut: no `;`, terminator follows directly.
            after_sel
        } else {
            return Err(Osc52ParseError::MissingPayloadSeparator);
        };

        // Strip terminator. Accept ESC \ (2 bytes) or BEL (1 byte).
        let payload_b64 = if let Some(rest) = payload_with_term.strip_suffix(b"\x1b\\") {
            rest
        } else if let Some(rest) = payload_with_term.strip_suffix(b"\x07") {
            rest
        } else if payload_with_term.ends_with(b"\x1b\\") || payload_with_term.ends_with(b"\x07") {
            // already handled above
            payload_with_term
        } else {
            return Err(Osc52ParseError::MissingTerminator);
        };

        let operation = if selector.is_read_request() {
            Osc52Operation::Read(selector)
        } else if payload_b64.is_empty() {
            Osc52Operation::Clear(selector)
        } else {
            let payload_str = std::str::from_utf8(payload_b64)
                .map_err(|e| Osc52ParseError::InvalidBase64(format!("payload not utf-8: {e}")))?;
            let data = base64_decode(payload_str)?;
            Osc52Operation::Write { selector, data }
        };

        Ok(Self { operation })
    }
}

/// Minimal RFC 4648 §4 base64 codec.
///
/// We avoid pulling in the `base64` crate to keep this protocol layer
/// dependency-light — the wire alphabet is small and stable, and
/// correctness is testable. If performance ever matters, swap the
/// internals for `base64::engine::general_purpose::STANDARD`.
fn base64_encode(input: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let mut out = String::with_capacity(input.len().div_ceil(3) * 4);
    let mut chunks = input.chunks_exact(3);
    for chunk in &mut chunks {
        let b0 = chunk[0];
        let b1 = chunk[1];
        let b2 = chunk[2];
        out.push(ALPHABET[(b0 >> 2) as usize] as char);
        out.push(ALPHABET[((b0 & 0x03) << 4 | (b1 >> 4)) as usize] as char);
        out.push(ALPHABET[((b1 & 0x0F) << 2 | (b2 >> 6)) as usize] as char);
        out.push(ALPHABET[(b2 & 0x3F) as usize] as char);
    }
    let rem = chunks.remainder();
    match rem.len() {
        0 => {}
        1 => {
            let b0 = rem[0];
            out.push(ALPHABET[(b0 >> 2) as usize] as char);
            out.push(ALPHABET[((b0 & 0x03) << 4) as usize] as char);
            out.push('=');
            out.push('=');
        }
        2 => {
            let b0 = rem[0];
            let b1 = rem[1];
            out.push(ALPHABET[(b0 >> 2) as usize] as char);
            out.push(ALPHABET[((b0 & 0x03) << 4 | (b1 >> 4)) as usize] as char);
            out.push(ALPHABET[((b1 & 0x0F) << 2) as usize] as char);
            out.push('=');
        }
        _ => unreachable!("chunks_exact remainder is at most 2"),
    }
    out
}

fn base64_decode(input: &str) -> Result<Vec<u8>, Osc52ParseError> {
    fn val(b: u8) -> Option<u8> {
        match b {
            b'A'..=b'Z' => Some(b - b'A'),
            b'a'..=b'z' => Some(b - b'a' + 26),
            b'0'..=b'9' => Some(b - b'0' + 52),
            b'+' => Some(62),
            b'/' => Some(63),
            _ => None,
        }
    }

    let bytes = input.as_bytes();
    if bytes.is_empty() {
        return Ok(Vec::new());
    }
    if bytes.len() % 4 != 0 {
        return Err(Osc52ParseError::InvalidBase64(format!(
            "length {} not a multiple of 4",
            bytes.len()
        )));
    }

    let mut out = Vec::with_capacity(bytes.len() / 4 * 3);
    let chunks: Vec<&[u8]> = bytes.chunks(4).collect();
    let last_idx = chunks.len().saturating_sub(1);
    for (idx, chunk) in chunks.iter().copied().enumerate() {
        // Padding (`=`) is only legal at positions 2 / 3 of the *last* chunk.
        let pad2 = chunk[2] == b'=';
        let pad3 = chunk[3] == b'=';
        if (pad2 || pad3) && idx != last_idx {
            return Err(Osc52ParseError::InvalidBase64(
                "padding byte in non-final chunk".into(),
            ));
        }
        // Within the last chunk, positions 0 and 1 must never be `=`
        // (RFC 4648 §3.5); and if position 2 is `=` then position 3 must
        // also be `=` (else padding leaks into the data).
        if idx == last_idx {
            if chunk[0] == b'=' || chunk[1] == b'=' {
                return Err(Osc52ParseError::InvalidBase64(
                    "padding byte in data position".into(),
                ));
            }
            if pad2 && !pad3 {
                return Err(Osc52ParseError::InvalidBase64(
                    "non-padding chars after padding".into(),
                ));
            }
        }

        let c0 = val(chunk[0]).ok_or_else(|| {
            Osc52ParseError::InvalidBase64(format!("bad base64 char {:#x}", chunk[0]))
        })?;
        let c1 = val(chunk[1]).ok_or_else(|| {
            Osc52ParseError::InvalidBase64(format!("bad base64 char {:#x}", chunk[1]))
        })?;
        let c2 = if pad2 {
            0
        } else {
            val(chunk[2]).ok_or_else(|| {
                Osc52ParseError::InvalidBase64(format!("bad base64 char {:#x}", chunk[2]))
            })?
        };
        let c3 = if pad3 {
            0
        } else {
            val(chunk[3]).ok_or_else(|| {
                Osc52ParseError::InvalidBase64(format!("bad base64 char {:#x}", chunk[3]))
            })?
        };

        out.push((c0 << 2) | (c1 >> 4));
        if !pad2 {
            out.push((c1 << 4) | (c2 >> 2));
        }
        if !pad3 {
            out.push((c2 << 6) | c3);
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selector_roundtrip_bytes() {
        for s in [
            Osc52Selector::Clipboard,
            Osc52Selector::Primary,
            Osc52Selector::Secondary,
            Osc52Selector::ReadRequest,
        ] {
            assert_eq!(Osc52Selector::from_byte(s.as_byte()), Some(s));
        }
        assert_eq!(Osc52Selector::from_byte(b'x'), None);
    }

    #[test]
    fn selector_display_matches_as_byte() {
        for s in [
            Osc52Selector::Clipboard,
            Osc52Selector::Primary,
            Osc52Selector::Secondary,
            Osc52Selector::ReadRequest,
        ] {
            let s_ascii = s.to_string();
            assert_eq!(s_ascii.as_bytes()[0], s.as_byte());
        }
    }

    #[test]
    fn base64_encode_known_vectors() {
        // RFC 4648 §10 test vectors
        assert_eq!(base64_encode(b""), "");
        assert_eq!(base64_encode(b"f"), "Zg==");
        assert_eq!(base64_encode(b"fo"), "Zm8=");
        assert_eq!(base64_encode(b"foo"), "Zm9v");
        assert_eq!(base64_encode(b"foob"), "Zm9vYg==");
        assert_eq!(base64_encode(b"fooba"), "Zm9vYmE=");
        assert_eq!(base64_encode(b"foobar"), "Zm9vYmFy");
    }

    #[test]
    fn base64_decode_known_vectors() {
        assert_eq!(base64_decode(""), Ok(Vec::<u8>::new()));
        assert_eq!(base64_decode("Zg=="), Ok(b"f".to_vec()));
        assert_eq!(base64_decode("Zm8="), Ok(b"fo".to_vec()));
        assert_eq!(base64_decode("Zm9v"), Ok(b"foo".to_vec()));
        assert_eq!(base64_decode("Zm9vYg=="), Ok(b"foob".to_vec()));
        assert_eq!(base64_decode("Zm9vYmE="), Ok(b"fooba".to_vec()));
        assert_eq!(base64_decode("Zm9vYmFy"), Ok(b"foobar".to_vec()));
    }

    #[test]
    fn base64_roundtrip_random() {
        // Try every byte-triple including padding boundaries.
        for len in 0..=20 {
            let data: Vec<u8> = (0..len as u8).map(|i| i.wrapping_mul(37)).collect();
            let encoded = base64_encode(&data);
            let decoded = base64_decode(&encoded).expect("decode should succeed");
            assert_eq!(decoded, data, "len={len}");
        }
    }

    #[test]
    fn serialize_write_hello() {
        let c = Osc52Clipboard::write("hello");
        let bytes = c.serialize().expect("write serializes");
        // Expected: ESC ] 52 ; c ; aGVsbG8= ESC \
        let expected = b"\x1b]52;c;aGVsbG8=\x1b\\";
        assert_eq!(bytes, expected);
    }

    #[test]
    fn parse_write_hello_roundtrip() {
        let original = Osc52Clipboard::write("hello");
        let bytes = original.serialize().unwrap();
        let parsed = Osc52Clipboard::parse(&bytes).unwrap();
        assert_eq!(parsed, original);
    }

    #[test]
    fn serialize_clear() {
        let c = Osc52Clipboard::clear();
        let bytes = c.serialize().unwrap();
        assert_eq!(bytes, b"\x1b]52;c;\x1b\\");
    }

    #[test]
    fn parse_clear_roundtrip() {
        let original = Osc52Clipboard::clear();
        let bytes = original.serialize().unwrap();
        let parsed = Osc52Clipboard::parse(&bytes).unwrap();
        assert_eq!(parsed, original);
    }

    #[test]
    fn serialize_read_request_rejected() {
        let c = Osc52Clipboard::read();
        assert_eq!(
            c.serialize(),
            Err(Osc52Error::ReadRequestNotSerializable)
        );
    }

    #[test]
    fn parse_read_request_succeeds() {
        // `\x1b]52;?\x1b\\` — the `?` selector is itself a read request, and
        // there is no payload between the second `;` and the ST.
        let wire = b"\x1b]52;?\x1b\\";
        let parsed = Osc52Clipboard::parse(wire).unwrap();
        assert_eq!(
            parsed.operation,
            Osc52Operation::Read(Osc52Selector::ReadRequest)
        );
        assert!(parsed.is_read_request());
    }

    #[test]
    fn parse_with_bel_terminator() {
        let wire = b"\x1b]52;c;aGVsbG8=\x07";
        let parsed = Osc52Clipboard::parse(wire).unwrap();
        assert_eq!(parsed, Osc52Clipboard::write("hello"));
    }

    #[test]
    fn parse_non_ascii_payload_via_utf8_text() {
        // "héllo" → base64 "aMOpbGxv"
        let c = Osc52Clipboard::write("héllo".as_bytes().to_vec());
        let bytes = c.serialize().unwrap();
        let parsed = Osc52Clipboard::parse(&bytes).unwrap();
        assert_eq!(parsed, c);
    }

    #[test]
    fn parse_binary_payload() {
        // All 256 byte values, base64 of a 256-byte run.
        let data: Vec<u8> = (0..=255u8).collect();
        let c = Osc52Clipboard::write_to(Osc52Selector::Primary, data.clone());
        let bytes = c.serialize().unwrap();
        let parsed = Osc52Clipboard::parse(&bytes).unwrap();
        assert_eq!(parsed, c);
        if let Osc52Operation::Write { data: got, .. } = parsed.operation {
            assert_eq!(got, data);
        } else {
            panic!("expected Write");
        }
    }

    #[test]
    fn parse_primary_selector() {
        let c = Osc52Clipboard::write_to(Osc52Selector::Primary, "x");
        let bytes = c.serialize().unwrap();
        let parsed = Osc52Clipboard::parse(&bytes).unwrap();
        assert_eq!(parsed.selector(), Osc52Selector::Primary);
    }

    #[test]
    fn parse_secondary_selector() {
        let c = Osc52Clipboard::write_to(Osc52Selector::Secondary, "y");
        let bytes = c.serialize().unwrap();
        let parsed = Osc52Clipboard::parse(&bytes).unwrap();
        assert_eq!(parsed.selector(), Osc52Selector::Secondary);
    }

    #[test]
    fn parse_rejects_unknown_command() {
        let wire = b"\x1b]53;c;AAAA\x1b\\";
        assert!(matches!(
            Osc52Clipboard::parse(wire),
            Err(Osc52ParseError::UnknownCommand(53))
        ));
    }

    #[test]
    fn parse_rejects_missing_introducer() {
        assert!(matches!(
            Osc52Clipboard::parse(b"52;c;AAAA\x1b\\"),
            Err(Osc52ParseError::MissingIntroducer { .. })
        ));
    }

    #[test]
    fn parse_rejects_empty() {
        assert_eq!(Osc52Clipboard::parse(b""), Err(Osc52ParseError::Empty));
    }

    #[test]
    fn parse_rejects_invalid_selector() {
        let wire = b"\x1b]52;z;AAAA\x1b\\";
        assert!(matches!(
            Osc52Clipboard::parse(wire),
            Err(Osc52ParseError::InvalidSelector(b'z'))
        ));
    }

    #[test]
    fn parse_rejects_missing_terminator() {
        let wire = b"\x1b]52;c;aGVsbG8=";
        assert_eq!(
            Osc52Clipboard::parse(wire),
            Err(Osc52ParseError::MissingTerminator)
        );
    }

    #[test]
    fn parse_rejects_invalid_base64() {
        let wire = b"\x1b]52;c;!!!!\x1b\\";
        assert!(matches!(
            Osc52Clipboard::parse(wire),
            Err(Osc52ParseError::InvalidBase64(_))
        ));
    }
}
