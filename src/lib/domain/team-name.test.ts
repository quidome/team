import { describe, expect, it } from 'vitest';

import { normalizeTeamName, teamNamesMatch } from './team-name';

describe('normalizeTeamName', () => {
  it.each([
    ['Blue Drakes - M16-1', 'Blue Drakes M16-1'],
    ['Blue Drakes – M16-1', 'Blue Drakes M16-1'],
    ['Blue  Drakes   M16-1', 'Blue Drakes M16-1'],
    ['  Blue Drakes M16-1  ', 'Blue Drakes M16-1'],
  ])('normalizes %s to %s', (value, expected) => {
    expect(normalizeTeamName(value)).toBe(expected);
  });

  it('leaves a hyphen inside a single token untouched', () => {
    expect(normalizeTeamName('Blue Drakes M16-1')).toBe('Blue Drakes M16-1');
  });
});

describe('teamNamesMatch', () => {
  it('matches names that only differ by dash formatting', () => {
    expect(teamNamesMatch('Blue Drakes - M16-1', 'Blue Drakes M16-1')).toBe(true);
  });

  it('matches case-insensitively', () => {
    expect(teamNamesMatch('blue drakes m16-1', 'Blue Drakes M16-1')).toBe(true);
  });

  it('does not match genuinely different names', () => {
    expect(teamNamesMatch('Blue Drakes M16-1', 'Woodpeckers M16-2')).toBe(false);
  });
});
