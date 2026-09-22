import { describe, expect, it } from 'vitest';

import { InMemoryLocationRepository } from '../../adapters/in-memory-location-repository';
import { importLocations } from './import-locations';

describe('import locations', () => {
  it('creates a new location, updates a changed one, and leaves an unchanged one alone', async () => {
    const locations = new InMemoryLocationRepository([
      { name: 'Home court', travelMinutes: 0 },
      { name: 'Away court', travelMinutes: 20 },
    ]);

    const result = await importLocations(locations, [
      { name: 'Home court', sourceRow: 2, travelMinutes: 0 },
      { name: 'Away court', sourceRow: 3, travelMinutes: 25 },
      { name: 'North court', sourceRow: 4, travelMinutes: 35 },
    ]);

    expect(result.unchanged).toEqual([{ sourceRow: 2, travelMinutes: 0 }]);
    expect(result.updated).toEqual([
      { name: 'Away court', previousTravelMinutes: 20, sourceRow: 3, travelMinutes: 25 },
    ]);
    expect(result.imported).toEqual([{ name: 'North court', sourceRow: 4, travelMinutes: 35 }]);
    expect(result.failed).toEqual([]);

    await expect(locations.findByName('Away court')).resolves.toEqual({
      name: 'Away court',
      travelMinutes: 25,
    });
    await expect(locations.findByName('North court')).resolves.toEqual({
      name: 'North court',
      travelMinutes: 35,
    });
  });
});
