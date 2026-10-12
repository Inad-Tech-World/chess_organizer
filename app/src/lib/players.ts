import type { Player, PlayerRecord } from '$lib/api/types';

export function blankPlayer(): Player {
  return {
    id: 0,
    name: '',
    sex: null,
    title: null,
    federation: null,
    fide_id: null,
    national_id: null,
    birth_date: null,
    club: null,
    ratings: { fide_standard: null, fide_rapid: null, fide_blitz: null, national: null },
    k_factor: null,
    starting_rank: 0,
    status: { type: 'Active' },
    byes: {},
    joined_round: 1,
    late_entry_bye: 'Zero',
    team: null,
    acceleration: []
  };
}

export function playerFromRecord(r: PlayerRecord): Player {
  return {
    ...blankPlayer(),
    name: r.name,
    fide_id: r.fide_id,
    national_id: r.national_id,
    federation: r.federation,
    sex: r.sex,
    title: r.title,
    ratings: {
      fide_standard: r.rating_standard,
      fide_rapid: r.rating_rapid,
      fide_blitz: r.rating_blitz,
      national: r.national_rating
    },
    k_factor: r.k_factor,
    birth_date: r.birth_year != null ? String(r.birth_year) : null,
    club: r.club
  };
}

/** Deep copy for editing in a dialog without touching the shown state. */
export function clone<T>(x: T): T {
  return JSON.parse(JSON.stringify(x)) as T;
}
