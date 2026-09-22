import { env } from '$env/dynamic/private';

import type { Location } from '$lib/application/locations/location-repository';
import type { Season } from '$lib/application/seasons/season-repository';
import type { Team } from '$lib/application/teams/team-repository';
import type { CoordinatorSettings } from '$lib/application/settings/coordinator-settings-repository';
import {
  currentCoordinatorSettingsRepository,
  currentLocationRepository,
  currentSeasonRepository,
  currentTeamRepository,
} from '$lib/server/composition-root';

export interface AdminPageData {
  coordinatorSettings: CoordinatorSettings | undefined;
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
    return { coordinatorSettings: undefined, locations: [], seasons: [], teams: [] };
  }

  const [locations, seasons, teams, coordinatorSettings] = await Promise.all([
    currentLocationRepository().findAll(),
    currentSeasonRepository().findAll(),
    currentTeamRepository().findAll(),
    currentCoordinatorSettingsRepository().get(),
  ]);

  return { coordinatorSettings, locations, seasons, teams };
};
