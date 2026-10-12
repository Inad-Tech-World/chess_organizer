// The C.07 tie-break catalogue as offered in the UI. Mirrors
// crates/co-core/src/tiebreak_spec.rs (acronyms and phases).

import type { TiebreakKind } from '$lib/bindings/TiebreakKind';
import type { TiebreakSpec } from '$lib/bindings/TiebreakSpec';

type Scope = 'individual' | 'both' | 'team';

export const TIEBREAKS: { kind: TiebreakKind; acronym: string; scope: Scope; phase: number }[] = [
  { kind: 'DirectEncounter', acronym: 'DE', scope: 'individual', phase: 1 },
  { kind: 'GamesWithBlack', acronym: 'BPG', scope: 'individual', phase: 2 },
  { kind: 'WinsWithBlack', acronym: 'BWG', scope: 'individual', phase: 2 },
  { kind: 'RoundsElectedToPlay', acronym: 'REP', scope: 'individual', phase: 2 },
  { kind: 'SonnebornBerger', acronym: 'SB', scope: 'individual', phase: 1 },
  { kind: 'AverageRatingOfOpponents', acronym: 'ARO', scope: 'individual', phase: 2 },
  { kind: 'TournamentPerformanceRating', acronym: 'TPR', scope: 'individual', phase: 2 },
  { kind: 'PerfectTournamentPerformance', acronym: 'PTP', scope: 'individual', phase: 2 },
  { kind: 'AveragePerformanceRatingOfOpponents', acronym: 'APRO', scope: 'individual', phase: 2 },
  { kind: 'AveragePerfectPerformanceOfOpponents', acronym: 'APPO', scope: 'individual', phase: 2 },
  { kind: 'Wins', acronym: 'WIN', scope: 'both', phase: 1 },
  { kind: 'WonGames', acronym: 'WON', scope: 'both', phase: 2 },
  { kind: 'ProgressiveScore', acronym: 'PS', scope: 'both', phase: 1 },
  { kind: 'Buchholz', acronym: 'BH', scope: 'both', phase: 1 },
  { kind: 'AverageOpponentsBuchholz', acronym: 'AOB', scope: 'both', phase: 2 },
  { kind: 'ForeBuchholz', acronym: 'FB', scope: 'both', phase: 2 },
  { kind: 'Koya', acronym: 'KS', scope: 'both', phase: 2 },
  { kind: 'BoardCount', acronym: 'BC', scope: 'team', phase: 3 },
  { kind: 'TopBoardResult', acronym: 'TBR', scope: 'team', phase: 3 },
  { kind: 'BottomBoardElimination', acronym: 'BBE', scope: 'team', phase: 3 },
  { kind: 'MatchPointsVsGamePoints', acronym: 'MPvGP', scope: 'team', phase: 3 },
  { kind: 'TeamSbMatchMatch', acronym: 'EMMSB', scope: 'team', phase: 3 },
  { kind: 'TeamSbMatchGame', acronym: 'EMGSB', scope: 'team', phase: 3 },
  { kind: 'TeamSbGameMatch', acronym: 'EGMSB', scope: 'team', phase: 3 },
  { kind: 'TeamSbGameGame', acronym: 'EGGSB', scope: 'team', phase: 3 },
  { kind: 'ExtendedDirectEncounter', acronym: 'EDE', scope: 'team', phase: 3 },
  { kind: 'EdeBoardCountTopBoard', acronym: 'EDEBT', scope: 'team', phase: 3 },
  { kind: 'EdeBoardCountBottomBoard', acronym: 'EDEBB', scope: 'team', phase: 3 },
  { kind: 'EdeTopBoard', acronym: 'EDET', scope: 'team', phase: 3 },
  { kind: 'EdeBottomBoard', acronym: 'EDEB', scope: 'team', phase: 3 },
  { kind: 'StrengthCompensatedScore', acronym: 'SSSC', scope: 'team', phase: 3 },
  { kind: 'SecondaryScore', acronym: 'OTHER_SEC', scope: 'team', phase: 3 }
];

export function tiebreakInfo(kind: TiebreakKind) {
  return TIEBREAKS.find((tb) => tb.kind === kind)!;
}

/** Same rank order descriptor as `TiebreakSpec::label` in Rust, e.g. `BH/C1`. */
export function tiebreakLabel(spec: TiebreakSpec): string {
  const m = spec.modifiers;
  let s = tiebreakInfo(spec.kind).acronym;
  if (m.team_score === 'MatchPoints') s += ':MP';
  if (m.team_score === 'GamePoints') s += ':GP';
  if (m.cut_lowest > 0) s += m.cut_lowest === m.cut_highest ? `/M${m.cut_lowest}` : `/C${m.cut_lowest}`;
  if (m.koya_limit != null) s += `/L${m.koya_limit >= 0 ? '+' : ''}${m.koya_limit}`;
  if (m.fore) s += '/F';
  if (m.forfeits_as_played) s += '/P';
  if (m.sssc_factor != null) s += `/K${m.sssc_factor}`;
  return s;
}

export function plainSpec(kind: TiebreakKind): TiebreakSpec {
  return {
    kind,
    modifiers: {
      cut_lowest: 0,
      cut_highest: 0,
      koya_limit: null,
      sssc_factor: null,
      forfeits_as_played: false,
      fore: false,
      team_score: null
    }
  };
}
