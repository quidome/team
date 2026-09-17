import type { Location, LocationRepository } from './location-repository';

export const configureLocation = async (
  locations: LocationRepository,
  location: Location,
): Promise<Location> => {
  const existingLocation = await locations.findByName(location.name);

  if (existingLocation) {
    return existingLocation;
  }

  return locations.save(location);
};
