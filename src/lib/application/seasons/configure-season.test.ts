import { describe, expect, it } from 'vitest';

import { InMemorySeasonRepository } from '../../adapters/in-memory-season-repository';
import { configureSeason } from './configure-season';

describe('configure season', () => {
  it('stores a season ending in the year after its starting year', async () => {
    const seasons = new InMemorySeasonRepository();

    const season = await configureSeason(seasons, 2026);

    expect(season).toEqual({ endingYear: 2027, startingYear: 2026 });
    await expect(seasons.findByStartingYear(2026)).resolves.toEqual(season);
  });

  it('returns an already-configured season rather than creating a duplicate', async () => {
    const seasons = new InMemorySeasonRepository([{ endingYear: 2027, startingYear: 2026 }]);

    await expect(configureSeason(seasons, 2026)).resolves.toEqual({
      endingYear: 2027,
      startingYear: 2026,
    });
  });
});
