// Keyboard result entry, as arbiters type it on the pairings screen:
//   1 = 1-0   0 = 0-1   5 or = = ½-½
//   + = +/- (white wins by forfeit)   - = -/+   * = -/- (double forfeit)
//   Delete / Backspace clears the result.

import type { GameResult } from '$lib/bindings/GameResult';
import type { SideResult } from '$lib/bindings/SideResult';

const r = (white: SideResult, black: SideResult): GameResult => ({ white, black });

export const WHITE_WINS = r('Win', 'Loss');
export const DRAW = r('Draw', 'Draw');
export const BLACK_WINS = r('Loss', 'Win');
export const WHITE_FORFEIT_WIN = r('ForfeitWin', 'ForfeitLoss');
export const BLACK_FORFEIT_WIN = r('ForfeitLoss', 'ForfeitWin');
export const DOUBLE_FORFEIT = r('ForfeitLoss', 'ForfeitLoss');

const KEYS: Record<string, GameResult> = {
  '1': WHITE_WINS,
  '0': BLACK_WINS,
  '5': DRAW,
  '=': DRAW,
  '½': DRAW,
  '+': WHITE_FORFEIT_WIN,
  '-': BLACK_FORFEIT_WIN,
  '*': DOUBLE_FORFEIT
};

/**
 * The result a key enters: a `GameResult`, `null` to clear the result, or
 * `undefined` if the key is not a result key.
 */
export function resultForKey(key: string): GameResult | null | undefined {
  if (key === 'Delete' || key === 'Backspace') return null;
  return KEYS[key];
}

function side(s: SideResult): string {
  switch (s) {
    case 'Win':
    case 'UnratedWin':
      return '1';
    case 'Draw':
    case 'UnratedDraw':
      return '½';
    case 'Loss':
    case 'UnratedLoss':
      return '0';
    case 'ForfeitWin':
      return '+';
    case 'ForfeitLoss':
      return '-';
  }
}

/** Display code matching `GameResult::code` in Rust: `1-0`, `½-½`, `+/-`. */
export function resultCode(result: GameResult | null): string {
  if (!result) return '';
  const forfeit = [result.white, result.black].some((s) => s.startsWith('Forfeit'));
  return `${side(result.white)}${forfeit ? '/' : '-'}${side(result.black)}`;
}
