import { describe, expect, it } from 'vitest';
import { plainSpec, tiebreakLabel } from './tiebreaks';

describe('tiebreakLabel', () => {
  it('matches the Rust rank order descriptors', () => {
    const bh = plainSpec('Buchholz');
    expect(tiebreakLabel(bh)).toBe('BH');
    bh.modifiers.cut_lowest = 1;
    expect(tiebreakLabel(bh)).toBe('BH/C1');
    bh.modifiers.cut_highest = 1;
    bh.modifiers.forfeits_as_played = true;
    bh.modifiers.team_score = 'GamePoints';
    expect(tiebreakLabel(bh)).toBe('BH:GP/M1/P');
    const ks = plainSpec('Koya');
    ks.modifiers.koya_limit = 1;
    expect(tiebreakLabel(ks)).toBe('KS/L+1');
  });
});
