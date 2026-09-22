import { describe, expect, it } from 'vitest';

import { deriveSeasonHalf } from './season-half';

describe('deriveSeasonHalf', () => {
  it('maps a date in the season starting year to H1', () => {
    expect(deriveSeasonHalf('2026-09-26', 2026)).toBe('H1');
  });

  it('maps a date in the season starting year plus one to H2', () => {
    expect(deriveSeasonHalf('2027-01-09', 2026)).toBe('H2');
  });

  it('returns undefined for a date in neither calendar year of the season', () => {
    expect(deriveSeasonHalf('2025-12-31', 2026)).toBeUndefined();
    expect(deriveSeasonHalf('2028-01-01', 2026)).toBeUndefined();
  });
});
