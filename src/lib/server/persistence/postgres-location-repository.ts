import { asc, eq } from 'drizzle-orm';

import type { Location, LocationRepository } from '../../application/locations/location-repository';
import type { DatabaseConnection } from './database';
import { locations } from './schema';

const columns = {
  address: locations.address,
  name: locations.name,
  travelMinutes: locations.travelMinutes,
};

const toLocation = (row: {
  address: string | null;
  name: string;
  travelMinutes: number;
}): Location => ({
  ...(row.address ? { address: row.address } : {}),
  name: row.name,
  travelMinutes: row.travelMinutes,
});

export const createPostgresLocationRepository = (
  database: DatabaseConnection,
): LocationRepository => ({
  async findAll(): Promise<Location[]> {
    const rows = await database.select(columns).from(locations).orderBy(asc(locations.name));

    return rows.map(toLocation);
  },

  async findByName(name: string): Promise<Location | undefined> {
    const [row] = await database
      .select(columns)
      .from(locations)
      .where(eq(locations.name, name))
      .limit(1);

    return row ? toLocation(row) : undefined;
  },

  async save(location: Location): Promise<Location> {
    const [row] = await database
      .insert(locations)
      .values({
        address: location.address ?? null,
        name: location.name,
        travelMinutes: location.travelMinutes,
      })
      .returning(columns);

    if (!row) {
      throw new Error('PostgreSQL did not return the stored location');
    }

    return toLocation(row);
  },

  async updateName(currentName: string, location: Location): Promise<Location> {
    const [row] = await database
      .update(locations)
      .set({
        address: location.address ?? null,
        name: location.name,
        travelMinutes: location.travelMinutes,
      })
      .where(eq(locations.name, currentName))
      .returning(columns);

    if (!row) {
      throw new Error('Location does not exist');
    }

    return toLocation(row);
  },

  async deleteByName(name: string): Promise<void> {
    await database.delete(locations).where(eq(locations.name, name));
  },
});
