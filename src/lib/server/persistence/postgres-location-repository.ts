import { asc, eq } from 'drizzle-orm';

import type { Location, LocationRepository } from '../../application/locations/location-repository';
import type { DatabaseConnection } from './database';
import { locations } from './schema';

export const createPostgresLocationRepository = (
  database: DatabaseConnection,
): LocationRepository => ({
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

  async updateName(currentName: string, location: Location): Promise<Location> {
    const [updatedLocation] = await database
      .update(locations)
      .set(location)
      .where(eq(locations.name, currentName))
      .returning({ name: locations.name, travelMinutes: locations.travelMinutes });

    if (!updatedLocation) {
      throw new Error('Location does not exist');
    }

    return updatedLocation;
  },

  async deleteByName(name: string): Promise<void> {
    await database.delete(locations).where(eq(locations.name, name));
  },
});
