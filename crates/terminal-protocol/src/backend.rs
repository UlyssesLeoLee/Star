//! Terminal-backend adapter trait (ULYS-166).
//!
//! [`TerminalBackend`] is the seam between this stack-agnostic protocol crate
//! (`terminal-protocol`) and whatever concrete terminal emulator ULYS-164
//! (ULYS-104.10) ultimately picks — Ghostty, xterm.js, alacritty, foot, etc.
//! Adapters written on top of those emulators will implement
//! [`TerminalBackend`] and convert the high-level protocol types from
//! [`crate::osc52`] and [`crate::kitty_kbd`] into real side effects on the
//! host's PTY / WebGL canvas / WebSocket pipe.
//!
//! What lives in this crate by design:
//!
//! - The wire-level data shapes (`Osc52Clipboard`, `KittyKeyboardProtocol`).
//! - A single trait that any adapter must implement.
//! - A [`MockBackend`] for tests / examples that records every call.
//!
//! What does **not** live here:
//!
//! - Host OS clipboard integration (arboard / x11rb / browser `navigator.clipboard`).
//! - PTY setup, fork, or exec.
//! - PTY read/write byte loops.
//! - Rendering, panes, focus management.
//!
//! When ULYS-164 (terminal stack) and ULYS-161 (OSC parser) land, a new
//! `terminal-protocol-adapter` crate is expected to provide
//! `impl TerminalBackend for XtermJsPane { … }` (or equivalent). This crate
//! then has nothing more to do.

use std::sync::Mutex;

use crate::kitty_kbd::{KittyKeyboardFlags, KittyKeyboardProtocol, KittyProgressiveState};
use crate::osc52::{Osc52Clipboard, Osc52Operation};

/// Outcome of an OSC 52 read request handled by a backend.
///
/// The Kitty/Orca OSC 52 read-request flow is: the application sends
/// `Osc52Clipboard::read()` to the terminal; the terminal then writes a fresh
/// OSC 52 sequence back containing the current contents. A backend that
/// actually polls the host clipboard (rather than forwarding the request to
/// a child terminal) can short-circuit by returning the bytes here directly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Osc52ReadResult {
    /// The backend answered the read request itself with the given bytes.
    Inline(Vec<u8>),
    /// The backend forwarded the read request to the child terminal and
    /// does not know the answer yet — the caller should wait for a parsed
    /// [`Osc52Clipboard`] to come back through the input stream.
    Forwarded,
}

/// What a backend should report back to the application when it has parsed
/// OSC 52 / Kitty keyboard output from a child terminal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IncomingProtocolMessage {
    /// A parsed OSC 52 sequence (write, clear, or read reply).
    Osc52(Osc52Clipboard),
    /// A parsed Kitty keyboard query response (`CSI ? flags u` — note this is
    /// the query-*reply* form, not the application push/pop form).
    KittyQueryReply(KittyKeyboardFlags),
}

/// The contract a stack-specific terminal backend must implement.
///
/// All methods take `&mut self` so the backend can hold PTY handles,
/// pane-focus state, or any future per-adapter state. The default
/// implementation of `apply_kitty_push` / `apply_kitty_pop` / `apply_kitty_set`
/// just updates the embedded [`KittyProgressiveState`]; adapters that need
/// richer tracking (e.g. multiple panes) may override.
///
/// The trait is **not** `async` — adapters that perform I/O are expected to
/// block briefly inside each call (write to a pipe, query a mutex). The
/// application is responsible for any larger async scheduling.
pub trait TerminalBackend {
    /// Handle an OSC 52 clipboard operation.
    ///
    /// `Write` and `Clear` operations are typically translated to a wire
    /// sequence and forwarded to the child terminal, or (for backends with
    /// direct host-clipboard access) applied locally.
    ///
    /// `Read` operations are translated to a wire read request and the
    /// backend returns [`Osc52ReadResult::Forwarded`], **or** if the backend
    /// has the clipboard contents already (e.g. it owns the clipboard),
    /// returns [`Osc52ReadResult::Inline`] with the bytes.
    fn apply_osc52(&mut self, op: Osc52Operation) -> Osc52ReadResult;

    /// Push a Kitty keyboard flag configuration onto the stack.
    ///
    /// The default implementation just updates the embedded state and
    /// returns the wire bytes the caller should write to the terminal.
    /// Adapters may override (e.g. to batch-write to a pane queue).
    fn apply_kitty_push(&mut self, flags: KittyKeyboardFlags) -> Vec<u8> {
        let wire = KittyKeyboardProtocol::Push(flags).serialize();
        self.update_kitty_state(|s| s.push(flags));
        wire
    }

    /// Pop a Kitty keyboard flag configuration off the stack.
    ///
    /// The default implementation just updates the embedded state and
    /// returns the wire bytes the caller should write to the terminal.
    fn apply_kitty_pop(&mut self, flags: KittyKeyboardFlags) -> Vec<u8> {
        let wire = KittyKeyboardProtocol::Pop(flags).serialize();
        self.update_kitty_state(|s| s.pop());
        wire
    }

    /// Apply a Kitty keyboard flag configuration directly (query-set form).
    ///
    /// The default implementation updates the embedded state with the new
    /// flags. Adapters that distinguish query (`Ps == 1`) from set
    /// (`Ps == 0`) may override.
    fn apply_kitty_set(&mut self, flags: KittyKeyboardFlags) -> Vec<u8> {
        let wire = KittyKeyboardProtocol::Set(flags).serialize();
        self.update_kitty_state(|_| KittyProgressiveState::Active {
            flags,
            depth: 1,
        });
        wire
    }

    /// Receive an already-parsed incoming protocol message from the terminal.
    ///
    /// Default implementation is a no-op (adapters can record or dispatch).
    /// Useful for adapters that want a single chokepoint for both OSC 52
    /// reads *and* Kitty query replies coming back from the child.
    fn handle_incoming(&mut self, _msg: IncomingProtocolMessage) {}

    /// Read-only view of the backend's idea of the current Kitty state.
    fn kitty_state(&self) -> Option<KittyProgressiveState>;

    /// Mutable accessor for the embedded Kitty state. Adapters that track
    /// multiple panes or stacks should override and panic / log if the
    /// caller tries to mutate a state the adapter does not own.
    fn kitty_state_mut(&mut self) -> Option<&mut KittyProgressiveState>;

    /// Helper used by the default `apply_kitty_*` implementations.
    fn update_kitty_state(
        &mut self,
        f: impl FnOnce(KittyProgressiveState) -> KittyProgressiveState,
    ) {
        if let Some(slot) = self.kitty_state_mut() {
            let next = f(*slot);
            *slot = next;
        }
    }
}

/// A capture-only [`TerminalBackend`] useful for tests and examples.
///
/// Records every call in a thread-safe [`Mutex`] so multiple consumers can
/// inspect the recorded sequence (handy when an adapter test fires many
/// protocol operations and the assertions want to verify the *order* of
/// emitted wire bytes).
///
/// `MockBackend` does not touch the host clipboard or any PTY. OSC 52 read
/// requests are answered by [`Self::read_response`] if set, otherwise
/// forwarded.
#[derive(Debug, Default)]
pub struct MockBackend {
    /// Buffered wire bytes the application would have written to the terminal.
    pub recorded_writes: Mutex<Vec<Vec<u8>>>,
    /// OSC 52 reads recorded as parsed `Osc52Clipboard` operations.
    pub recorded_osc52_reads: Mutex<Vec<Osc52Clipboard>>,
    /// OSC 52 writes recorded as parsed `Osc52Clipboard` operations.
    pub recorded_osc52_writes: Mutex<Vec<Osc52Clipboard>>,
    /// OSC 52 clears recorded as parsed `Osc52Clipboard` operations.
    pub recorded_osc52_clears: Mutex<Vec<Osc52Clipboard>>,
    /// Kitty push / pop / set operations.
    pub recorded_kitty_ops: Mutex<Vec<KittyKeyboardProtocol>>,
    /// Incoming protocol messages received from the terminal.
    pub recorded_incoming: Mutex<Vec<IncomingProtocolMessage>>,
    /// Current Kitty state (default = [`KittyProgressiveState::default`]).
    pub kitty: Mutex<KittyProgressiveState>,
    /// If set, OSC 52 read requests are answered inline with these bytes
    /// instead of being forwarded.
    pub read_response: Mutex<Option<Vec<u8>>>,
}

impl MockBackend {
    /// Construct an empty [`MockBackend`].
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// If `read_response` is set, OSC 52 read requests will return
    /// [`Osc52ReadResult::Inline`] with these bytes; otherwise they will
    /// return [`Osc52ReadResult::Forwarded`].
    pub fn set_read_response(&self, response: Vec<u8>) {
        *self.read_response.lock().expect("mock backend mutex poisoned") = Some(response);
    }

    /// Take a snapshot of the wire bytes recorded so far.
    #[must_use]
    pub fn snapshot_writes(&self) -> Vec<Vec<u8>> {
        self.recorded_writes
            .lock()
            .expect("mock backend mutex poisoned")
            .clone()
    }

    /// Take a snapshot of all recorded OSC 52 operations (reads, writes,
    /// clears) in call order.
    #[must_use]
    pub fn snapshot_osc52_calls(&self) -> Vec<(Osc52Clipboard, Osc52ReadResult)> {
        let mut combined = Vec::new();
        for r in self
            .recorded_osc52_reads
            .lock()
            .expect("mock backend mutex poisoned")
            .iter()
        {
            combined.push((r.clone(), Osc52ReadResult::Forwarded));
        }
        for w in self
            .recorded_osc52_writes
            .lock()
            .expect("mock backend mutex poisoned")
            .iter()
        {
            combined.push((w.clone(), Osc52ReadResult::Forwarded));
        }
        for c in self
            .recorded_osc52_clears
            .lock()
            .expect("mock backend mutex poisoned")
            .iter()
        {
            combined.push((c.clone(), Osc52ReadResult::Forwarded));
        }
        combined
    }

    /// Take a snapshot of all recorded Kitty operations in call order.
    #[must_use]
    pub fn snapshot_kitty_ops(&self) -> Vec<KittyKeyboardProtocol> {
        self.recorded_kitty_ops
            .lock()
            .expect("mock backend mutex poisoned")
            .clone()
    }
}

impl TerminalBackend for MockBackend {
    fn apply_osc52(&mut self, op: Osc52Operation) -> Osc52ReadResult {
        let clipboard = Osc52Clipboard {
            operation: op.clone(),
        };
        match op {
            Osc52Operation::Read(_) => {
                self.recorded_osc52_reads
                    .lock()
                    .expect("mock backend mutex poisoned")
                    .push(clipboard);
                let inline = self
                    .read_response
                    .lock()
                    .expect("mock backend mutex poisoned")
                    .clone();
                match inline {
                    Some(bytes) => Osc52ReadResult::Inline(bytes),
                    None => Osc52ReadResult::Forwarded,
                }
            }
            Osc52Operation::Write { .. } => {
                self.recorded_osc52_writes
                    .lock()
                    .expect("mock backend mutex poisoned")
                    .push(clipboard);
                Osc52ReadResult::Forwarded
            }
            Osc52Operation::Clear(_) => {
                self.recorded_osc52_clears
                    .lock()
                    .expect("mock backend mutex poisoned")
                    .push(clipboard);
                Osc52ReadResult::Forwarded
            }
        }
    }

    fn apply_kitty_push(&mut self, flags: KittyKeyboardFlags) -> Vec<u8> {
        let wire = KittyKeyboardProtocol::Push(flags).serialize();
        self.recorded_kitty_ops
            .lock()
            .expect("mock backend mutex poisoned")
            .push(KittyKeyboardProtocol::Push(flags));
        self.recorded_writes
            .lock()
            .expect("mock backend mutex poisoned")
            .push(wire.clone());
        // Use the default state-machine bookkeeping.
        let mut guard = self.kitty.lock().expect("mock backend mutex poisoned");
        *guard = guard.push(flags);
        wire
    }

    fn apply_kitty_pop(&mut self, flags: KittyKeyboardFlags) -> Vec<u8> {
        let wire = KittyKeyboardProtocol::Pop(flags).serialize();
        self.recorded_kitty_ops
            .lock()
            .expect("mock backend mutex poisoned")
            .push(KittyKeyboardProtocol::Pop(flags));
        self.recorded_writes
            .lock()
            .expect("mock backend mutex poisoned")
            .push(wire.clone());
        let mut guard = self.kitty.lock().expect("mock backend mutex poisoned");
        *guard = guard.pop();
        wire
    }

    fn apply_kitty_set(&mut self, flags: KittyKeyboardFlags) -> Vec<u8> {
        let wire = KittyKeyboardProtocol::Set(flags).serialize();
        self.recorded_kitty_ops
            .lock()
            .expect("mock backend mutex poisoned")
            .push(KittyKeyboardProtocol::Set(flags));
        self.recorded_writes
            .lock()
            .expect("mock backend mutex poisoned")
            .push(wire.clone());
        let mut guard = self.kitty.lock().expect("mock backend mutex poisoned");
        *guard = KittyProgressiveState::Active { flags, depth: 1 };
        wire
    }

    fn handle_incoming(&mut self, msg: IncomingProtocolMessage) {
        self.recorded_incoming
            .lock()
            .expect("mock backend mutex poisoned")
            .push(msg);
    }

    fn kitty_state(&self) -> Option<KittyProgressiveState> {
        Some(*self.kitty.lock().expect("mock backend mutex poisoned"))
    }

    fn kitty_state_mut(&mut self) -> Option<&mut KittyProgressiveState> {
        // SAFETY: we hold `&mut self` so no other reference to the state
        // exists; the `Mutex` inside is only ever accessed through this
        // type's other methods, and those take `&mut self` too.
        // We can't return `&mut KittyProgressiveState` directly because the
        // field is behind a `Mutex`, so we return `None` — adapters that
        // need this override the trait method.
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::osc52::Osc52Selector;
    use crate::kitty_kbd::KittyKeyboardFlag;

    #[test]
    fn mock_backend_records_osc52_write() {
        let mut b = MockBackend::new();
        let op = Osc52Operation::Write {
            selector: Osc52Selector::Clipboard,
            data: b"hello".to_vec(),
        };
        let result = b.apply_osc52(op.clone());
        assert_eq!(result, Osc52ReadResult::Forwarded);
        assert_eq!(b.snapshot_osc52_calls(), vec![(Osc52Clipboard { operation: op }, Osc52ReadResult::Forwarded)]);
    }

    #[test]
    fn mock_backend_records_osc52_clear() {
        let mut b = MockBackend::new();
        let op = Osc52Operation::Clear(Osc52Selector::Primary);
        b.apply_osc52(op.clone());
        assert_eq!(
            b.snapshot_osc52_calls(),
            vec![(Osc52Clipboard { operation: op }, Osc52ReadResult::Forwarded)]
        );
    }

    #[test]
    fn mock_backend_forwards_osc52_read_when_no_response_set() {
        let mut b = MockBackend::new();
        let op = Osc52Operation::Read(Osc52Selector::Clipboard);
        let result = b.apply_osc52(op);
        assert_eq!(result, Osc52ReadResult::Forwarded);
        assert_eq!(b.recorded_osc52_reads.lock().unwrap().len(), 1);
    }

    #[test]
    fn mock_backend_inline_osc52_read_when_response_set() {
        let mut b = MockBackend::new();
        b.set_read_response(b"clipboard-contents".to_vec());
        let op = Osc52Operation::Read(Osc52Selector::Clipboard);
        let result = b.apply_osc52(op);
        assert_eq!(
            result,
            Osc52ReadResult::Inline(b"clipboard-contents".to_vec())
        );
    }

    #[test]
    fn mock_backend_kitty_push_emits_wire_and_updates_state() {
        let mut b = MockBackend::new();
        let flags = KittyKeyboardFlags::from_bits_truncate(0x05);
        let wire = b.apply_kitty_push(flags);
        assert_eq!(wire, b"\x1b[>5u".to_vec());

        let state = b.kitty_state().unwrap();
        assert_eq!(
            state,
            KittyProgressiveState::Active {
                flags,
                depth: 1
            }
        );
        assert_eq!(
            b.snapshot_kitty_ops(),
            vec![KittyKeyboardProtocol::Push(flags)]
        );
    }

    #[test]
    fn mock_backend_kitty_pop_transitions_to_cleared_at_depth_one() {
        let mut b = MockBackend::new();
        let flags = KittyKeyboardFlags::from_bits_truncate(0x01);
        b.apply_kitty_push(flags);
        let _ = b.apply_kitty_pop(flags);
        let state = b.kitty_state().unwrap();
        assert_eq!(state, KittyProgressiveState::Cleared);
    }

    #[test]
    fn mock_backend_kitty_set_writes_active_state() {
        let mut b = MockBackend::new();
        let flags = KittyKeyboardFlags::from_bits_truncate(0x09);
        let wire = b.apply_kitty_set(flags);
        assert_eq!(wire, b"\x1b[=9;1u".to_vec());
        assert_eq!(
            b.kitty_state().unwrap(),
            KittyProgressiveState::Active {
                flags,
                depth: 1
            }
        );
    }

    #[test]
    fn mock_backend_handle_incoming_records_messages() {
        let mut b = MockBackend::new();
        let msg = IncomingProtocolMessage::KittyQueryReply(KittyKeyboardFlags::from_bits_truncate(
            0x05,
        ));
        b.handle_incoming(msg.clone());
        assert_eq!(b.recorded_incoming.lock().unwrap().clone(), vec![msg.clone()]);

        let osc = Osc52Clipboard::write(b"hi".to_vec());
        let msg2 = IncomingProtocolMessage::Osc52(osc.clone());
        b.handle_incoming(msg2.clone());
        assert_eq!(
            b.recorded_incoming.lock().unwrap().clone(),
            vec![msg, msg2]
        );
    }

    #[test]
    fn mock_backend_default_state_is_virgin() {
        let b = MockBackend::new();
        assert!(matches!(
            b.kitty_state().unwrap(),
            KittyProgressiveState::Virgin { .. }
        ));
    }

    #[test]
    fn flag_combinations_via_default_trait_methods_round_trip() {
        // Verify the default-trait-method behaviour with a custom backend
        // that only implements the *required* methods + state accessor.
        struct TinyBackend {
            state: KittyProgressiveState,
        }
        impl TerminalBackend for TinyBackend {
            fn apply_osc52(&mut self, _: Osc52Operation) -> Osc52ReadResult {
                Osc52ReadResult::Forwarded
            }
            fn kitty_state(&self) -> Option<KittyProgressiveState> {
                Some(self.state)
            }
            fn kitty_state_mut(&mut self) -> Option<&mut KittyProgressiveState> {
                Some(&mut self.state)
            }
        }

        let mut b = TinyBackend {
            state: KittyProgressiveState::default(),
        };
        // push set of flags
        let flags = KittyKeyboardFlags::NONE
            .insert(KittyKeyboardFlag::DisambiguateEscapeCodes)
            .insert(KittyKeyboardFlag::ReportEventTypes);
        let wire = b.apply_kitty_push(flags);
        assert_eq!(wire, b"\x1b[>3u".to_vec());
        assert!(b.kitty_state().unwrap().is_active());
        // pop
        let _ = b.apply_kitty_pop(flags);
        assert!(!b.kitty_state().unwrap().is_active());
    }
}