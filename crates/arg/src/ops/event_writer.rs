// SPDX-License-Identifier: MIT OR Apache-2.0
//! `EventWriter` (per DD-AGENT-RELATIONSHIP-001 §3.1).
//!
//! Append-only writer for [`ARGEvent`]. The current placeholder keeps
//! events in an in-memory `Vec` so unit tests can verify the
//! ordering and the count. The real driver call (Transaction class,
//! per 守门 #13 b) lands in P3-C W1.

use std::sync::Mutex;

use crate::models::ARGEvent;

/// Append-only event writer (Transaction class, per 守门 #13 b).
#[derive(Debug, Default)]
pub struct EventWriter {
    inner: Mutex<Vec<ARGEvent>>,
}

impl EventWriter {
    /// Build a new empty writer.
    pub fn new() -> Self {
        Self::default()
    }

    /// Append an event.
    pub async fn append(&self, event: ARGEvent) -> Result<(), crate::error::ARGError> {
        let mut guard = self.inner.lock().expect("event writer mutex poisoned");
        guard.push(event);
        Ok(())
    }

    /// Snapshot of the events written so far. Used by tests.
    pub fn snapshot(&self) -> Vec<ARGEvent> {
        self.inner
            .lock()
            .expect("event writer mutex poisoned")
            .clone()
    }

    /// Number of events written so far.
    pub fn len(&self) -> usize {
        self.inner
            .lock()
            .expect("event writer mutex poisoned")
            .len()
    }

    /// `true` iff no event has been written yet.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
