import type { GameImportContext } from '$lib/application/imports/import-games';
import {
  currentCoordinatorSettingsRepository,
  currentLocationRepository,
  currentTeamRepository,
} from '$lib/server/composition-root';

export const buildGameImportContext = async (): Promise<GameImportContext> => {
  const [teams, locations] = await Promise.all([
    currentTeamRepository().findAll(),
    currentLocationRepository().findAll(),
  ]);

  return {
    knownLocationTravelMinutes: Object.fromEntries(
      locations.map((location) => [location.name, location.travelMinutes]),
    ),
    knownTeamNames: teams.map((team) => team.name),
  };
};

export const currentImportPrimaryTeamName = async (): Promise<string | undefined> =>
  (await currentCoordinatorSettingsRepository().get())?.primaryTeamName;
