import type { Location, LocationRepository } from '../application/locations/location-repository';

export class InMemoryLocationRepository implements LocationRepository {
  private readonly locations = new Map<string, Location>();

  constructor(initialLocations: Location[] = []) {
    for (const location of initialLocations) {
      this.locations.set(location.name, location);
    }
  }

  async findAll(): Promise<Location[]> {
    return [...this.locations.values()].sort((left, right) => left.name.localeCompare(right.name));
  }

  async findByName(name: string): Promise<Location | undefined> {
    return this.locations.get(name);
  }

  async save(location: Location): Promise<Location> {
    this.locations.set(location.name, location);

    return location;
  }
}
