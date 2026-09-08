// SPDX-License-Identifier: MIT OR Apache-2.0
//! `AchievementOps` (per DD-AGENT-RELATIONSHIP-001 §3.1 + §4).
//!
//! 1 method (`unlock`) for the public surface; the full CRUD is on the
//! P3-C W1 backlog (`G-1`). The `unlock` method is **idempotent** per
//! `(user_id, achievement_code)` — a duplicate call returns
//! `Ok(false)` and does not append a new event.

use std::sync::Mutex;

use crate::error::ARGError;
use crate::models::achievement_unlock::AchievementUnlock;
use crate::models::ARGEvent;

use super::event_writer::EventWriter;

/// Achievement unlock + query operations.
#[derive(Debug)]
pub struct AchievementOps {
    writer: EventWriter,
    /// Local idempotency cache: (user_id, achievement_code) → already unlocked?
    seen: Mutex<std::collections::HashSet<(uuid::Uuid, String)>>,
}

impl AchievementOps {
    /// Build a new ops struct.
    pub fn new(writer: EventWriter) -> Self {
        Self {
            writer,
            seen: Mutex::new(std::collections::HashSet::new()),
        }
    }

    /// Unlock an achievement for a given user. Idempotent per
    /// `(user_id, achievement_code)` (per DD §3.3.5).
    ///
    /// Returns `Ok(true)` on the first unlock, `Ok(false)` on duplicates.
    pub async fn unlock(&self, unlock: AchievementUnlock) -> Result<bool, ARGError> {
        let key = (unlock.user_id, unlock.achievement_code.clone());
        {
            let mut seen = self.seen.lock().expect("achievement seen mutex poisoned");
            if !seen.insert(key) {
                return Ok(false);
            }
        }
        self.writer
            .append(ARGEvent::AchievementUnlocked(unlock))
            .await?;
        Ok(true)
    }

    /// How many `(user, code)` pairs are currently cached as "unlocked".
    pub fn unique_unlocks(&self) -> usize {
        self.seen
            .lock()
            .expect("achievement seen mutex poisoned")
            .len()
    }
}
