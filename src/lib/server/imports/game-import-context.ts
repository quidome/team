import type { GameImportContext } from '$lib/application/imports/import-games';
import {
  currentCoordinatorSettingsRepository,
  currentTeamRepository,
} from '$lib/server/composition-root';

export const buildGameImportContext = async (): Promise<GameImportContext> => {
  const [teams, settings] = await Promise.all([
    currentTeamRepository().findAll(),
    currentCoordinatorSettingsRepository().get(),
  ]);

  return {
    knownTeamNames: teams.map((team) => team.name),
    ...(settings ? { season: { startingYear: settings.seasonStartingYear } } : {}),
  };
};

export const currentImportPrimaryTeamName = async (): Promise<string | undefined> =>
  (await currentCoordinatorSettingsRepository().get())?.primaryTeamName;
