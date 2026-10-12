import { describe, expect, it } from 'vitest';
import { BLACK_WINS, DRAW, resultCode, resultForKey, WHITE_FORFEIT_WIN } from './results';

describe('result keys', () => {
  it('maps arbiter keys to results', () => {
    expect(resultForKey('1')).toEqual({ white: 'Win', black: 'Loss' });
    expect(resultForKey('0')).toEqual(BLACK_WINS);
    expect(resultForKey('5')).toEqual(DRAW);
    expect(resultForKey('=')).toEqual(DRAW);
    expect(resultForKey('+')).toEqual(WHITE_FORFEIT_WIN);
  });

  it('clears with Delete and ignores other keys', () => {
    expect(resultForKey('Delete')).toBeNull();
    expect(resultForKey('Backspace')).toBeNull();
    expect(resultForKey('x')).toBeUndefined();
  });

  it('formats codes like the Rust side', () => {
    expect(resultCode(DRAW)).toBe('½-½');
    expect(resultCode(WHITE_FORFEIT_WIN)).toBe('+/-');
    expect(resultCode({ white: 'ForfeitLoss', black: 'ForfeitLoss' })).toBe('-/-');
    expect(resultCode(null)).toBe('');
  });
});
