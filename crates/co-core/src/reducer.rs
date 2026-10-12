//! Applies events to a tournament, checking integrity as it goes.

use crate::error::RuleError;
use crate::event::Event;
use crate::ids::{PlayerId, RoundNo};
use crate::model::{ByeKind, Pairing, PlayerStatus, Round, TeamMatch, Tournament};
use std::collections::HashSet;

impl Tournament {
    /// Builds a tournament from its first event, which must be `TournamentCreated`.
    pub fn create(event: &Event) -> Result<Tournament, RuleError> {
        match event {
            Event::TournamentCreated {
                id,
                info,
                settings,
                kind,
            } => Ok(Tournament {
                id: *id,
                info: info.clone(),
                settings: settings.clone(),
                kind: *kind,
                players: Vec::new(),
                teams: Vec::new(),
                rounds: Vec::new(),
                prize_categories: Vec::new(),
            }),
            Event::TournamentImported { tournament, .. } => Ok((**tournament).clone()),
            _ => Err(RuleError::NotCreated),
        }
    }

    /// Applies one event. On error the tournament is left unchanged.
    pub fn apply(&mut self, event: &Event) -> Result<(), RuleError> {
        match event {
            Event::TournamentCreated { .. } | Event::TournamentImported { .. } => {
                Err(RuleError::AlreadyCreated)
            }
            Event::InfoUpdated { info } => {
                self.info = info.clone();
                Ok(())
            }
            Event::SettingsUpdated { settings } => {
                if settings.rounds < self.paired_rounds() {
                    return Err(RuleError::RoundsBelowPlayed {
                        requested: settings.rounds,
                        played: self.paired_rounds(),
                    });
                }
                for group in &settings.forbidden_groups {
                    self.require_players(group.members.iter().copied())?;
                }
                self.settings = settings.clone();
                Ok(())
            }
            Event::KindChanged { kind } => {
                if !self.rounds.is_empty() {
                    return Err(RuleError::KindLocked);
                }
                self.kind = *kind;
                Ok(())
            }

            Event::PlayerAdded { player } => {
                match self.players.binary_search_by_key(&player.id, |p| p.id) {
                    Ok(_) => Err(RuleError::DuplicatePlayer(player.id)),
                    Err(pos) => {
                        if let Some(team) = player.team {
                            self.require_team(team)?;
                        }
                        self.players.insert(pos, player.clone());
                        Ok(())
                    }
                }
            }
            Event::PlayerUpdated { player } => {
                if let Some(team) = player.team {
                    self.require_team(team)?;
                }
                let slot = self
                    .player_mut(player.id)
                    .ok_or(RuleError::UnknownPlayer(player.id))?;
                *slot = player.clone();
                Ok(())
            }
            Event::PlayerRemoved { player } => {
                self.require_players([*player])?;
                if self.has_games(*player) {
                    return Err(RuleError::PlayerHasGames(*player));
                }
                self.players.retain(|p| p.id != *player);
                for team in &mut self.teams {
                    team.members.retain(|m| m != player);
                }
                for group in &mut self.settings.forbidden_groups {
                    group.members.retain(|m| m != player);
                }
                Ok(())
            }
            Event::PlayerWithdrawn { player, from_round } => {
                let p = self
                    .player_mut(*player)
                    .ok_or(RuleError::UnknownPlayer(*player))?;
                p.status = PlayerStatus::Withdrawn {
                    from_round: *from_round,
                };
                Ok(())
            }
            Event::PlayerReentered { player } => {
                let p = self
                    .player_mut(*player)
                    .ok_or(RuleError::UnknownPlayer(*player))?;
                p.status = PlayerStatus::Active;
                Ok(())
            }
            Event::ByeSet {
                player,
                round,
                kind,
            } => {
                if *round <= self.paired_rounds() {
                    return Err(RuleError::Other(format!(
                        "round {round} is already paired; edit its pairings instead"
                    )));
                }
                if *round > self.settings.rounds {
                    return Err(RuleError::TooManyRounds(self.settings.rounds));
                }
                if *kind == Some(ByeKind::PairingAllocated) {
                    return Err(RuleError::Other(
                        "a pairing-allocated bye cannot be requested".into(),
                    ));
                }
                let p = self
                    .player_mut(*player)
                    .ok_or(RuleError::UnknownPlayer(*player))?;
                match kind {
                    Some(k) => p.byes.insert(*round, *k),
                    None => p.byes.remove(round),
                };
                Ok(())
            }
            Event::StartingRanksSet { order } => {
                self.require_players(order.iter().copied())?;
                let listed: HashSet<PlayerId> = order.iter().copied().collect();
                let mut rest: Vec<(u32, PlayerId)> = self
                    .players
                    .iter()
                    .filter(|p| !listed.contains(&p.id))
                    .map(|p| (p.starting_rank, p.id))
                    .collect();
                rest.sort();
                let full = order
                    .iter()
                    .copied()
                    .chain(rest.into_iter().map(|(_, id)| id));
                for (rank, id) in full.enumerate() {
                    if let Some(p) = self.player_mut(id) {
                        p.starting_rank = rank as u32 + 1;
                    }
                }
                Ok(())
            }

            Event::RoundPaired {
                round,
                pairings,
                matches,
                method,
            } => {
                let expected = self.paired_rounds() + 1;
                if *round != expected {
                    return Err(RuleError::RoundOutOfSequence {
                        expected,
                        got: *round,
                    });
                }
                if *round > self.settings.rounds {
                    return Err(RuleError::TooManyRounds(self.settings.rounds));
                }
                self.validate_round(*round, pairings, matches)?;
                self.rounds.push(Round {
                    number: *round,
                    pairings: pairings.clone(),
                    matches: matches.clone(),
                    method: *method,
                });
                Ok(())
            }
            Event::PairingsEdited { round, pairings } => {
                let matches = self
                    .round(*round)
                    .ok_or(RuleError::UnknownRound(*round))?
                    .matches
                    .clone();
                self.validate_round(*round, pairings, &matches)?;
                if let Some(r) = self.round_mut(*round) {
                    r.pairings = pairings.clone();
                }
                Ok(())
            }
            Event::RoundDeleted { round } => {
                if *round == 0 || *round != self.paired_rounds() {
                    return Err(RuleError::NotLastRound(self.paired_rounds()));
                }
                self.rounds.pop();
                Ok(())
            }
            Event::ResultEntered {
                round,
                board,
                result,
            } => {
                let r = self
                    .round_mut(*round)
                    .ok_or(RuleError::UnknownRound(*round))?;
                let game = r
                    .pairings
                    .iter_mut()
                    .find(|p| p.board() == Some(*board))
                    .ok_or(RuleError::UnknownBoard {
                        round: *round,
                        board: *board,
                    })?;
                if let Pairing::Game { result: slot, .. } = game {
                    *slot = *result;
                }
                Ok(())
            }

            Event::TeamAdded { team } => {
                if self.team(team.id).is_some() {
                    return Err(RuleError::DuplicateTeam(team.id));
                }
                self.require_players(team.members.iter().copied())?;
                self.teams.push(team.clone());
                Ok(())
            }
            Event::TeamUpdated { team } => {
                self.require_players(team.members.iter().copied())?;
                let slot = self
                    .teams
                    .iter_mut()
                    .find(|t| t.id == team.id)
                    .ok_or(RuleError::UnknownTeam(team.id))?;
                *slot = team.clone();
                Ok(())
            }
            Event::TeamRemoved { team } => {
                self.require_team(*team)?;
                let in_matches = self
                    .rounds
                    .iter()
                    .flat_map(|r| &r.matches)
                    .any(|m| m.home == *team || m.away == Some(*team));
                if in_matches {
                    return Err(RuleError::Other(format!(
                        "team {team} has already been paired"
                    )));
                }
                self.teams.retain(|t| t.id != *team);
                for p in &mut self.players {
                    if p.team == Some(*team) {
                        p.team = None;
                    }
                }
                Ok(())
            }
            Event::LineupSet {
                round,
                match_number,
                team,
                lineup,
            } => {
                self.require_players(lineup.iter().copied())?;
                let m = self.team_match_mut(*round, *match_number)?;
                if m.home == *team {
                    m.home_lineup = lineup.clone();
                } else if m.away == Some(*team) {
                    m.away_lineup = lineup.clone();
                } else {
                    return Err(RuleError::UnknownTeam(*team));
                }
                m.rebuild_games();
                Ok(())
            }
            Event::TeamGameResultEntered {
                round,
                match_number,
                board,
                result,
            } => {
                let m = self.team_match_mut(*round, *match_number)?;
                let game = m
                    .games
                    .iter_mut()
                    .find(|p| p.board() == Some(*board))
                    .ok_or(RuleError::UnknownBoard {
                        round: *round,
                        board: *board,
                    })?;
                if let Pairing::Game { result: slot, .. } = game {
                    *slot = *result;
                }
                Ok(())
            }

            Event::PrizeCategoryAdded { category } => {
                if self.prize_categories.iter().any(|c| c.id == category.id) {
                    return Err(RuleError::Other(format!(
                        "prize category {} already exists",
                        category.id
                    )));
                }
                self.prize_categories.push(category.clone());
                Ok(())
            }
            Event::PrizeCategoryUpdated { category } => {
                let slot = self
                    .prize_categories
                    .iter_mut()
                    .find(|c| c.id == category.id)
                    .ok_or(RuleError::UnknownPrizeCategory(category.id.0))?;
                *slot = category.clone();
                Ok(())
            }
            Event::PrizeCategoryRemoved { category } => {
                let before = self.prize_categories.len();
                self.prize_categories.retain(|c| c.id != *category);
                if self.prize_categories.len() == before {
                    return Err(RuleError::UnknownPrizeCategory(category.0));
                }
                Ok(())
            }

            Event::Undo { .. } | Event::Redo { .. } => Err(RuleError::Other(
                "undo/redo markers are resolved by history::replay".into(),
            )),
        }
    }

    fn require_players(&self, ids: impl IntoIterator<Item = PlayerId>) -> Result<(), RuleError> {
        for id in ids {
            if self.player(id).is_none() {
                return Err(RuleError::UnknownPlayer(id));
            }
        }
        Ok(())
    }

    fn require_team(&self, id: crate::ids::TeamId) -> Result<(), RuleError> {
        self.team(id).map(|_| ()).ok_or(RuleError::UnknownTeam(id))
    }

    fn team_match_mut(
        &mut self,
        round: RoundNo,
        match_number: u16,
    ) -> Result<&mut TeamMatch, RuleError> {
        self.round_mut(round)
            .ok_or(RuleError::UnknownRound(round))?
            .matches
            .iter_mut()
            .find(|m| m.number == match_number)
            .ok_or(RuleError::Other(format!(
                "round {round} has no match {match_number}"
            )))
    }

    /// Structural checks shared by pairing and editing a round.
    fn validate_round(
        &self,
        round: RoundNo,
        pairings: &[Pairing],
        matches: &[TeamMatch],
    ) -> Result<(), RuleError> {
        let mut seen = HashSet::new();
        let mut boards = HashSet::new();
        for p in pairings {
            if let Pairing::Game {
                white,
                black,
                board,
                ..
            } = *p
            {
                if white == black {
                    return Err(RuleError::SelfPairing(white));
                }
                if !boards.insert(board) {
                    return Err(RuleError::DuplicateBoard(board));
                }
            }
            for id in p.players() {
                self.require_players([id])?;
                if !seen.insert(id) {
                    return Err(RuleError::PlayerPairedTwice(id, round));
                }
            }
        }
        let mut teams = HashSet::new();
        for m in matches {
            for team in std::iter::once(m.home).chain(m.away) {
                self.require_team(team)?;
                if !teams.insert(team) {
                    return Err(RuleError::Other(format!(
                        "team {team} appears more than once in round {round}"
                    )));
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::*;
    use uuid::Uuid;

    fn created() -> Tournament {
        Tournament::create(&Event::TournamentCreated {
            id: Uuid::nil(),
            info: TournamentInfo::default(),
            settings: Settings::default(),
            kind: TournamentKind::default(),
        })
        .unwrap()
    }

    fn with_players(n: u32) -> Tournament {
        let mut t = created();
        for i in 1..=n {
            t.apply(&Event::PlayerAdded {
                player: Player::new(PlayerId(i), format!("Player {i}")),
            })
            .unwrap();
        }
        t
    }

    fn game(board: u16, w: u32, b: u32) -> Pairing {
        Pairing::Game {
            board,
            white: PlayerId(w),
            black: PlayerId(b),
            result: None,
        }
    }

    fn pair_round(
        t: &mut Tournament,
        round: RoundNo,
        pairings: Vec<Pairing>,
    ) -> Result<(), RuleError> {
        t.apply(&Event::RoundPaired {
            round,
            pairings,
            matches: vec![],
            method: PairingMethod::Manual,
        })
    }

    #[test]
    fn rejects_duplicate_player() {
        let mut t = with_players(2);
        let err = t
            .apply(&Event::PlayerAdded {
                player: Player::new(PlayerId(1), "Again"),
            })
            .unwrap_err();
        assert_eq!(err, RuleError::DuplicatePlayer(PlayerId(1)));
    }

    #[test]
    fn rounds_must_be_paired_in_order() {
        let mut t = with_players(4);
        let err = pair_round(&mut t, 2, vec![]).unwrap_err();
        assert_eq!(
            err,
            RuleError::RoundOutOfSequence {
                expected: 1,
                got: 2
            }
        );
        pair_round(&mut t, 1, vec![game(1, 1, 2), game(2, 3, 4)]).unwrap();
        assert_eq!(t.paired_rounds(), 1);
    }

    #[test]
    fn rejects_structurally_invalid_pairings() {
        let mut t = with_players(4);
        assert_eq!(
            pair_round(&mut t, 1, vec![game(1, 1, 1)]).unwrap_err(),
            RuleError::SelfPairing(PlayerId(1))
        );
        assert_eq!(
            pair_round(&mut t, 1, vec![game(1, 1, 2), game(2, 2, 3)]).unwrap_err(),
            RuleError::PlayerPairedTwice(PlayerId(2), 1)
        );
        assert_eq!(
            pair_round(&mut t, 1, vec![game(1, 1, 2), game(1, 3, 4)]).unwrap_err(),
            RuleError::DuplicateBoard(1)
        );
        assert_eq!(
            pair_round(&mut t, 1, vec![game(1, 1, 9)]).unwrap_err(),
            RuleError::UnknownPlayer(PlayerId(9))
        );
        assert!(t.rounds.is_empty());
    }

    #[test]
    fn enters_results_and_completes_round() {
        let mut t = with_players(4);
        pair_round(&mut t, 1, vec![game(1, 1, 2), game(2, 3, 4)]).unwrap();
        assert_eq!(t.completed_rounds(), 0);
        for board in [1, 2] {
            t.apply(&Event::ResultEntered {
                round: 1,
                board,
                result: Some(GameResult::DRAW),
            })
            .unwrap();
        }
        assert_eq!(t.completed_rounds(), 1);
        assert!(matches!(
            t.apply(&Event::ResultEntered {
                round: 1,
                board: 7,
                result: None
            }),
            Err(RuleError::UnknownBoard { round: 1, board: 7 })
        ));
    }

    #[test]
    fn cannot_remove_player_with_games() {
        let mut t = with_players(2);
        pair_round(&mut t, 1, vec![game(1, 1, 2)]).unwrap();
        assert_eq!(
            t.apply(&Event::PlayerRemoved {
                player: PlayerId(1)
            })
            .unwrap_err(),
            RuleError::PlayerHasGames(PlayerId(1))
        );
    }

    #[test]
    fn starting_ranks_follow_given_order() {
        let mut t = with_players(3);
        t.apply(&Event::StartingRanksSet {
            order: vec![PlayerId(3)],
        })
        .unwrap();
        let ranks: Vec<_> = t.players.iter().map(|p| p.starting_rank).collect();
        assert_eq!(ranks, vec![2, 3, 1]);
    }

    #[test]
    fn kind_locked_after_first_round() {
        let mut t = with_players(2);
        pair_round(&mut t, 1, vec![game(1, 1, 2)]).unwrap();
        assert_eq!(
            t.apply(&Event::KindChanged {
                kind: TournamentKind::RoundRobin { double: false }
            })
            .unwrap_err(),
            RuleError::KindLocked
        );
    }

    #[test]
    fn byes_only_for_future_rounds() {
        let mut t = with_players(2);
        pair_round(&mut t, 1, vec![game(1, 1, 2)]).unwrap();
        assert!(
            t.apply(&Event::ByeSet {
                player: PlayerId(1),
                round: 1,
                kind: Some(ByeKind::Half)
            })
            .is_err()
        );
        t.apply(&Event::ByeSet {
            player: PlayerId(1),
            round: 2,
            kind: Some(ByeKind::Half),
        })
        .unwrap();
        assert!(!t.player(PlayerId(1)).unwrap().is_available(2));
    }
}
