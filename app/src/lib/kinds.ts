import type { TournamentKind } from '$lib/api/types';

/**
 * Tournament types offered in the UI. `note` names a message shown in the
 * setup screen, e.g. for engines that have no endorsed reference to be
 * checked against.
 */
export const KINDS: { id: string; kind: TournamentKind; note?: string }[] = [
  { id: 'swiss-dutch', kind: { type: 'Swiss', system: 'Dutch' } },
  { id: 'swiss-dubov', kind: { type: 'Swiss', system: 'Dubov' }, note: 'setup-note-unverified' },
  { id: 'swiss-burstein', kind: { type: 'Swiss', system: 'Burstein' } },
  { id: 'swiss-lim', kind: { type: 'Swiss', system: 'Lim' }, note: 'setup-note-unverified' },
  { id: 'swiss-monrad', kind: { type: 'Swiss', system: 'Monrad' }, note: 'setup-note-monrad' },
  { id: 'rr', kind: { type: 'RoundRobin', double: false } },
  { id: 'rr-double', kind: { type: 'RoundRobin', double: true } },
  { id: 'team-swiss', kind: { type: 'TeamSwiss', system: 'Dutch' }, note: 'setup-note-team-swiss' },
  { id: 'team-rr', kind: { type: 'TeamRoundRobin', double: false } },
  { id: 'team-rr-double', kind: { type: 'TeamRoundRobin', double: true } }
];

export function kindId(kind: TournamentKind): string {
  return KINDS.find((k) => JSON.stringify(k.kind) === JSON.stringify(kind))?.id ?? 'swiss-dutch';
}

/** Message id naming a tournament type, e.g. `kind-swiss-dutch`. */
export function kindKey(kind: TournamentKind): string {
  return `kind-${kindId(kind)}`;
}

export function isTeamKind(kind: TournamentKind): boolean {
  return kind.type === 'TeamSwiss' || kind.type === 'TeamRoundRobin';
}
