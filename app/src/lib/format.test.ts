import { describe, expect, it } from 'vitest';
import { fmtPoints } from './format';

describe('fmtPoints', () => {
  it('uses halves like printed lists', () => {
    expect(fmtPoints(3)).toBe('3');
    expect(fmtPoints(2.5)).toBe('2½');
    expect(fmtPoints(0.5)).toBe('½');
    expect(fmtPoints(3.3)).toBe('3.3');
    expect(fmtPoints(null)).toBe('');
  });
});
