import type { GameImportContext } from '$lib/application/imports/import-games';
import {
  currentCoordinatorSettingsRepository,
  currentTeamRepository,
} from '$lib/server/composition-root';

export const buildGameImportContext = async (): Promise<GameImportContext> => {
  const teams = await currentTeamRepository().findAll();

  return {
    knownTeamNames: teams.map((team) => team.name),
  };
};

export const currentImportPrimaryTeamName = async (): Promise<string | undefined> =>
  (await currentCoordinatorSettingsRepository().get())?.primaryTeamName;
