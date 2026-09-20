import { asc, eq } from 'drizzle-orm';

import type { Location, LocationRepository } from '../../application/locations/location-repository';
import { createDatabase } from './database';
import { locations } from './schema';

type Database = ReturnType<typeof createDatabase>;

export const createPostgresLocationRepository = (database: Database): LocationRepository => ({
  async findAll(): Promise<Location[]> {
    return database
      .select({ name: locations.name, travelMinutes: locations.travelMinutes })
      .from(locations)
      .orderBy(asc(locations.name));
  },

  async findByName(name: string): Promise<Location | undefined> {
    const [location] = await database
      .select({ name: locations.name, travelMinutes: locations.travelMinutes })
      .from(locations)
      .where(eq(locations.name, name))
      .limit(1);

    return location;
  },

  async save(location: Location): Promise<Location> {
    const [storedLocation] = await database
      .insert(locations)
      .values(location)
      .returning({ name: locations.name, travelMinutes: locations.travelMinutes });

    if (!storedLocation) {
      throw new Error('PostgreSQL did not return the stored location');
    }

    return storedLocation;
  },
});
