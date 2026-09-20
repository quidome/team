import { env } from '$env/dynamic/private';

import type { Location } from '$lib/application/locations/location-repository';
import type { Season } from '$lib/application/seasons/season-repository';
import type { Team } from '$lib/application/teams/team-repository';
import {
  currentLocationRepository,
  currentSeasonRepository,
  currentTeamRepository,
} from '$lib/server/composition-root';

export interface AdminPageData {
  locations: Location[];
  seasons: Season[];
  teams: Team[];
}

export const loadLocations = async (): Promise<Location[]> => {
  if (!env.DATABASE_URL?.trim()) {
    return [];
  }

  return currentLocationRepository().findAll();
};

export const loadAdmin = async (): Promise<AdminPageData> => {
  if (!env.DATABASE_URL?.trim()) {
    return { locations: [], seasons: [], teams: [] };
  }

  const [locations, seasons, teams] = await Promise.all([
    currentLocationRepository().findAll(),
    currentSeasonRepository().findAll(),
    currentTeamRepository().findAll(),
  ]);

  return { locations, seasons, teams };
};
