import { eq } from 'drizzle-orm';
import { afterAll, beforeAll, describe, expect, it } from 'vitest';

import type { LocationRepository } from '../../application/locations/location-repository';
import { createDatabase } from './database';
import { createPostgresLocationRepository } from './postgres-location-repository';
import { locations } from './schema';

const databaseUrl = process.env.DATABASE_URL;

if (!databaseUrl) {
  describe.skip('PostgreSQL location repository', () => {});
} else {
  describe('PostgreSQL location repository', () => {
    const location = { name: 'Integration test court', travelMinutes: 20 };
    const database = createDatabase(databaseUrl);
    const repository: LocationRepository = createPostgresLocationRepository(database);

    beforeAll(async () => {
      await database.delete(locations).where(eq(locations.name, location.name));
    });

    afterAll(async () => {
      await database.delete(locations).where(eq(locations.name, location.name));
      await database.close();
    });

    it('persists and retrieves reusable locations', async () => {
      await expect(repository.save(location)).resolves.toEqual(location);
      await expect(repository.findByName(location.name)).resolves.toEqual(location);
      await expect(repository.findAll()).resolves.toContainEqual(location);
    });
  });
}
