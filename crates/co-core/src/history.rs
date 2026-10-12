//! Undo/redo over the append-only event log.
//!
//! Undo and redo are themselves events, so the audit trail is never rewritten.
//! Semantics match a text editor: `Undo` reverts the latest effective event,
//! `Redo` re-applies the latest undone one, and any new event clears the redo
//! stack.

use crate::error::{Error, RuleError};
use crate::event::{Envelope, Event};
use crate::model::Tournament;
use std::collections::HashSet;

/// Which events of a log are currently in effect.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HistoryCursor {
    /// Sequence numbers of effective events, in order.
    applied: Vec<u64>,
    /// Undone events, most recently undone last.
    undone: Vec<u64>,
}

impl HistoryCursor {
    /// Resolves the undo/redo markers of a log.
    pub fn from_log(log: &[Envelope]) -> Result<Self, RuleError> {
        let mut cursor = HistoryCursor::default();
        for env in log {
            cursor.push(env)?;
        }
        Ok(cursor)
    }

    /// Advances the cursor by one appended event.
    pub fn push(&mut self, env: &Envelope) -> Result<(), RuleError> {
        match env.event {
            Event::Undo { target } => {
                if self.next_undo() != Some(target) {
                    return Err(RuleError::HistoryMismatch(target));
                }
                self.applied.pop();
                self.undone.push(target);
            }
            Event::Redo { target } => {
                if self.next_redo() != Some(target) {
                    return Err(RuleError::HistoryMismatch(target));
                }
                self.undone.pop();
                self.applied.push(target);
            }
            _ => {
                self.applied.push(env.seq);
                self.undone.clear();
            }
        }
        Ok(())
    }

    /// The event an `Undo` would revert. The creation event cannot be undone.
    pub fn next_undo(&self) -> Option<u64> {
        if self.applied.len() > 1 {
            self.applied.last().copied()
        } else {
            None
        }
    }

    pub fn next_redo(&self) -> Option<u64> {
        self.undone.last().copied()
    }

    pub fn applied(&self) -> &[u64] {
        &self.applied
    }
}

/// Rebuilds the tournament from a full log, honouring undo/redo markers.
pub fn replay(log: &[Envelope]) -> Result<(Tournament, HistoryCursor), Error> {
    let cursor = HistoryCursor::from_log(log)?;
    let effective: HashSet<u64> = cursor.applied.iter().copied().collect();
    let mut events = log
        .iter()
        .filter(|e| effective.contains(&e.seq))
        .map(|e| &e.event);
    let first = events.next().ok_or(RuleError::NotCreated)?;
    let mut t = Tournament::create(first)?;
    for ev in events {
        t.apply(ev)?;
    }
    Ok((t, cursor))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::PlayerId;
    use crate::model::*;
    use chrono::Utc;
    use uuid::Uuid;

    struct Log(Vec<Envelope>);

    impl Log {
        fn push(&mut self, event: Event) -> u64 {
            let seq = self.0.len() as u64 + 1;
            self.0.push(Envelope {
                seq,
                at: Utc::now(),
                event,
            });
            seq
        }
    }

    fn base() -> Log {
        let mut log = Log(Vec::new());
        log.push(Event::TournamentCreated {
            id: Uuid::nil(),
            info: TournamentInfo::default(),
            settings: Settings::default(),
            kind: TournamentKind::default(),
        });
        log
    }

    fn add(log: &mut Log, id: u32) -> u64 {
        log.push(Event::PlayerAdded {
            player: Player::new(PlayerId(id), format!("P{id}")),
        })
    }

    #[test]
    fn undo_and_redo() {
        let mut log = base();
        add(&mut log, 1);
        let second = add(&mut log, 2);
        log.push(Event::Undo { target: second });
        let (t, cursor) = replay(&log.0).unwrap();
        assert_eq!(t.players.len(), 1);
        assert_eq!(cursor.next_redo(), Some(second));

        log.push(Event::Redo { target: second });
        let (t, cursor) = replay(&log.0).unwrap();
        assert_eq!(t.players.len(), 2);
        assert_eq!(cursor.next_redo(), None);
    }

    #[test]
    fn new_event_clears_redo() {
        let mut log = base();
        let first = add(&mut log, 1);
        log.push(Event::Undo { target: first });
        add(&mut log, 2);
        let (t, cursor) = replay(&log.0).unwrap();
        assert_eq!(cursor.next_redo(), None);
        assert_eq!(
            t.players.iter().map(|p| p.id.0).collect::<Vec<_>>(),
            vec![2]
        );
    }

    #[test]
    fn rejects_mismatched_markers() {
        let mut log = base();
        let first = add(&mut log, 1);
        add(&mut log, 2);
        log.push(Event::Undo { target: first });
        assert!(replay(&log.0).is_err());
    }

    #[test]
    fn creation_cannot_be_undone() {
        let log = base();
        let cursor = HistoryCursor::from_log(&log.0).unwrap();
        assert_eq!(cursor.next_undo(), None);
    }
}
