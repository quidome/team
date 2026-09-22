import { loadLocations } from '$lib/server/load-admin';
import { currentTeamSeasonContext } from '$lib/server/load-team';
import { loadProgram } from '$lib/server/load-program';

export const load = async () => {
  const [program, locations, context] = await Promise.all([
    loadProgram(),
    loadLocations(),
    currentTeamSeasonContext(),
  ]);

  return {
    ...program,
    locations,
    season: {
      endingYear: context.seasonStartingYear + 1,
      startingYear: context.seasonStartingYear,
    },
    teamName: context.teamName,
  };
};
