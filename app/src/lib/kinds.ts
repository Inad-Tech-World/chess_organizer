import type { TournamentKind } from '$lib/api/types';

/** Tournament types offered in the UI, with the phase each one's pairing lands in. */
export const KINDS: { id: string; kind: TournamentKind; phase: number }[] = [
  { id: 'swiss-dutch', kind: { type: 'Swiss', system: 'Dutch' }, phase: 1 },
  { id: 'swiss-dubov', kind: { type: 'Swiss', system: 'Dubov' }, phase: 4 },
  { id: 'swiss-burstein', kind: { type: 'Swiss', system: 'Burstein' }, phase: 4 },
  { id: 'swiss-lim', kind: { type: 'Swiss', system: 'Lim' }, phase: 4 },
  { id: 'swiss-monrad', kind: { type: 'Swiss', system: 'Monrad' }, phase: 4 },
  { id: 'rr', kind: { type: 'RoundRobin', double: false }, phase: 2 },
  { id: 'rr-double', kind: { type: 'RoundRobin', double: true }, phase: 2 },
  { id: 'team-swiss', kind: { type: 'TeamSwiss', system: 'Dutch' }, phase: 3 },
  { id: 'team-rr', kind: { type: 'TeamRoundRobin', double: false }, phase: 3 },
  { id: 'team-rr-double', kind: { type: 'TeamRoundRobin', double: true }, phase: 3 }
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
