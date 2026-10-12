// The C.07 tie-break catalogue as offered in the UI. Mirrors
// crates/co-core/src/tiebreak_spec.rs (acronyms).

import type { TiebreakKind } from '$lib/bindings/TiebreakKind';
import type { TiebreakSpec } from '$lib/bindings/TiebreakSpec';

type Scope = 'individual' | 'both' | 'team';

export const TIEBREAKS: { kind: TiebreakKind; acronym: string; scope: Scope }[] = [
  { kind: 'DirectEncounter', acronym: 'DE', scope: 'individual' },
  { kind: 'GamesWithBlack', acronym: 'BPG', scope: 'individual' },
  { kind: 'WinsWithBlack', acronym: 'BWG', scope: 'individual' },
  { kind: 'RoundsElectedToPlay', acronym: 'REP', scope: 'individual' },
  { kind: 'SonnebornBerger', acronym: 'SB', scope: 'individual' },
  { kind: 'AverageRatingOfOpponents', acronym: 'ARO', scope: 'individual' },
  { kind: 'TournamentPerformanceRating', acronym: 'TPR', scope: 'individual' },
  { kind: 'PerfectTournamentPerformance', acronym: 'PTP', scope: 'individual' },
  { kind: 'AveragePerformanceRatingOfOpponents', acronym: 'APRO', scope: 'individual' },
  { kind: 'AveragePerfectPerformanceOfOpponents', acronym: 'APPO', scope: 'individual' },
  { kind: 'Wins', acronym: 'WIN', scope: 'both' },
  { kind: 'WonGames', acronym: 'WON', scope: 'both' },
  { kind: 'ProgressiveScore', acronym: 'PS', scope: 'both' },
  { kind: 'Buchholz', acronym: 'BH', scope: 'both' },
  { kind: 'AverageOpponentsBuchholz', acronym: 'AOB', scope: 'both' },
  { kind: 'ForeBuchholz', acronym: 'FB', scope: 'both' },
  { kind: 'Koya', acronym: 'KS', scope: 'both' },
  { kind: 'BoardCount', acronym: 'BC', scope: 'team' },
  { kind: 'TopBoardResult', acronym: 'TBR', scope: 'team' },
  { kind: 'BottomBoardElimination', acronym: 'BBE', scope: 'team' },
  { kind: 'MatchPointsVsGamePoints', acronym: 'MPvGP', scope: 'team' },
  { kind: 'TeamSbMatchMatch', acronym: 'EMMSB', scope: 'team' },
  { kind: 'TeamSbMatchGame', acronym: 'EMGSB', scope: 'team' },
  { kind: 'TeamSbGameMatch', acronym: 'EGMSB', scope: 'team' },
  { kind: 'TeamSbGameGame', acronym: 'EGGSB', scope: 'team' },
  { kind: 'ExtendedDirectEncounter', acronym: 'EDE', scope: 'team' },
  { kind: 'EdeBoardCountTopBoard', acronym: 'EDEBT', scope: 'team' },
  { kind: 'EdeBoardCountBottomBoard', acronym: 'EDEBB', scope: 'team' },
  { kind: 'EdeTopBoard', acronym: 'EDET', scope: 'team' },
  { kind: 'EdeBottomBoard', acronym: 'EDEB', scope: 'team' },
  { kind: 'StrengthCompensatedScore', acronym: 'SSSC', scope: 'team' },
  { kind: 'SecondaryScore', acronym: 'OTHER_SEC', scope: 'team' }
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
