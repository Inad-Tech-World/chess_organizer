import type { Player } from '$lib/bindings/Player';
import type { RatingType } from '$lib/bindings/RatingType';

/** Points for display: `3`, `2½`, `½`, or a decimal for other point systems. */
export function fmtPoints(p: number | undefined | null): string {
  if (p == null) return '';
  const tenths = Math.round(p * 10);
  const whole = Math.trunc(tenths / 10);
  const rest = Math.abs(tenths % 10);
  if (rest === 0) return String(whole);
  if (rest === 5) return whole === 0 ? (tenths < 0 ? '-½' : '½') : `${whole}½`;
  return (tenths / 10).toFixed(1);
}

export function playerRating(p: Player, rt: RatingType): number | null {
  const r = p.ratings;
  const fide = rt === 'Standard' ? r.fide_standard : rt === 'Rapid' ? r.fide_rapid : r.fide_blitz;
  return fide ?? r.national ?? null;
}
