import { describe, expect, it } from 'vitest';

import { InMemoryLocationRepository } from '../../adapters/in-memory-location-repository';
import { configureLocation } from './configure-location';

describe('configure location', () => {
  it('stores a reusable location with travel time', async () => {
    const locations = new InMemoryLocationRepository();

    const location = await configureLocation(locations, {
      name: 'Home court',
      travelMinutes: 20,
    });

    expect(location).toEqual({ name: 'Home court', travelMinutes: 20 });
    await expect(locations.findByName('Home court')).resolves.toEqual(location);
  });

  it('lists reusable locations alphabetically', async () => {
    const locations = new InMemoryLocationRepository([
      { name: 'Away court', travelMinutes: 35 },
      { name: 'Home court', travelMinutes: 20 },
    ]);

    await expect(locations.findAll()).resolves.toEqual([
      { name: 'Away court', travelMinutes: 35 },
      { name: 'Home court', travelMinutes: 20 },
    ]);
  });

  it('returns an existing location rather than creating a duplicate', async () => {
    const locations = new InMemoryLocationRepository([{ name: 'Home court', travelMinutes: 20 }]);

    await expect(
      configureLocation(locations, { name: 'Home court', travelMinutes: 25 }),
    ).resolves.toEqual({ name: 'Home court', travelMinutes: 20 });
  });
});
