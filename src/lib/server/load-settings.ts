import { env } from '$env/dynamic/private';

import type { Location } from '$lib/application/locations/location-repository';
import { currentLocationRepository } from '$lib/server/composition-root';

export interface SettingsPageData {
  locations: Location[];
}

export const loadLocations = async (): Promise<Location[]> => {
  if (!env.DATABASE_URL?.trim()) {
    return [];
  }

  return currentLocationRepository().findAll();
};

export const loadSettings = async (): Promise<SettingsPageData> => ({
  locations: await loadLocations(),
});
