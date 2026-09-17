import { describe, expect, it } from 'vitest';

import { suggestDepartureTime } from './game';

describe('suggested game departure time', () => {
  it('subtracts travel time and the arrival buffer', () => {
    expect(suggestDepartureTime('14:30', 20, 30)).toBe('13:40');
  });

  it('wraps to the previous day when departure crosses midnight', () => {
    expect(suggestDepartureTime('00:15', 20, 30)).toBe('23:25');
  });
});
