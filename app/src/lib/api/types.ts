// Types shared with the Rust side. Domain types are generated from co-core
// by ts-rs (`cargo test -p co-core --features ts`); app-level DTOs are here.

export type { Tournament } from '$lib/bindings/Tournament';
export type { TournamentInfo } from '$lib/bindings/TournamentInfo';
export type { TournamentKind } from '$lib/bindings/TournamentKind';
export type { Settings } from '$lib/bindings/Settings';
export type { Player } from '$lib/bindings/Player';
export type { PlayerId } from '$lib/bindings/PlayerId';
export type { Pairing } from '$lib/bindings/Pairing';
export type { GameResult } from '$lib/bindings/GameResult';
export type { SideResult } from '$lib/bindings/SideResult';
export type { ByeKind } from '$lib/bindings/ByeKind';
export type { Team } from '$lib/bindings/Team';
export type { PrizeCategory } from '$lib/bindings/PrizeCategory';
export type { TiebreakKind } from '$lib/bindings/TiebreakKind';
export type { TiebreakSpec } from '$lib/bindings/TiebreakSpec';
export type { Title } from '$lib/bindings/Title';
export type { Sex } from '$lib/bindings/Sex';
export type { RatingType } from '$lib/bindings/RatingType';
export type { Round } from '$lib/bindings/Round';

import type { Tournament } from '$lib/bindings/Tournament';
import type { Sex } from '$lib/bindings/Sex';
import type { Title } from '$lib/bindings/Title';

export interface TournamentView {
  tournament: Tournament;
  path: string;
  can_undo: boolean;
  can_redo: boolean;
  undo_label: string | null;
  /** Scores after all paired rounds, keyed by player id. */
  scores: Record<string, number>;
  /** `round_scores[i]`: scores after round `i + 1`. */
  round_scores: Record<string, number>[];
  next_round: number;
}

export interface CommandError {
  code: 'planned' | 'rule' | 'no-tournament' | 'store' | 'io' | 'invalid';
  message: string;
  phase: number | null;
}

export interface PlayerRecord {
  fide_id: number | null;
  national_id: string | null;
  name: string;
  federation: string | null;
  sex: Sex | null;
  title: Title | null;
  rating_standard: number | null;
  rating_rapid: number | null;
  rating_blitz: number | null;
  national_rating: number | null;
  k_factor: number | null;
  birth_year: number | null;
  club: string | null;
}

export interface Source {
  id: number;
  name: string;
  period: string | null;
  imported_at: string;
  players: number;
}

export type ListKind =
  | { type: 'StartingRank' }
  | { type: 'Alphabetical' }
  | { type: 'Pairings'; round: number }
  | { type: 'Standings'; round: number }
  | { type: 'Crosstable'; round: number }
  | { type: 'Scorecards' }
  | { type: 'BoardCards'; round: number }
  | { type: 'Prizes' }
  | { type: 'TeamRosters' }
  | { type: 'RatingReport' };

export type ExportFormat = 'html' | 'pdf' | 'csv' | 'xlsx';

export interface ManualOutcome {
  applied: boolean;
  violations: string[];
  view: TournamentView;
}

export interface ImportResult {
  view: TournamentView;
  warnings: string[];
}
