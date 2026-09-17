import { describe, expect, it } from 'vitest';

import { calculateNormalAgeGroup } from './normal-age-group';

describe('normal age-group calculation', () => {
  it.each([
    [2012, 'U16'],
    [2011, 'U16'],
    [2010, 'U18'],
    [2009, 'U18'],
  ])('maps birth year %i to %s for the 2026–2027 season', (birthYear, expectedAgeGroup) => {
    expect(calculateNormalAgeGroup(2026, birthYear)).toBe(expectedAgeGroup);
  });
});
