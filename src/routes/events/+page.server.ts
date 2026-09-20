import { loadLocations } from '$lib/server/load-admin';
import { currentSeasonStartingYear, currentTeamName } from '$lib/server/load-team';
import { loadProgram } from '$lib/server/load-program';

export const load = async () => {
  const [program, locations] = await Promise.all([loadProgram(), loadLocations()]);

  return {
    ...program,
    locations,
    season: {
      endingYear: currentSeasonStartingYear + 1,
      startingYear: currentSeasonStartingYear,
    },
    teamName: currentTeamName,
  };
};
