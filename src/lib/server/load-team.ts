import { env } from '$env/dynamic/private';

import { defaultTeamSeasonContext } from '$lib/application/team-context';
import { readTeam, type TeamPlayer } from '$lib/application/team/read-team';
import { loadProgram } from '$lib/server/load-program';
import { currentMembershipRepository, currentPlayerRepository } from '$lib/server/composition-root';
import type { ProgramEvent } from '$lib/application/program/read-program';

export const currentSeasonStartingYear = defaultTeamSeasonContext.seasonStartingYear;
export const currentTeamName = defaultTeamSeasonContext.teamName;

export interface TeamPageData {
  events: ProgramEvent[];
  players: TeamPlayer[];
  selectedEventId?: string;
  season: {
    endingYear: number;
    startingYear: number;
  };
  teamName: string;
}

export const loadTeam = async (selectedEventId?: string): Promise<TeamPageData> => {
  if (!env.DATABASE_URL?.trim()) {
    return {
      events: [],
      players: [],
      selectedEventId,
      season: {
        endingYear: currentSeasonStartingYear + 1,
        startingYear: currentSeasonStartingYear,
      },
      teamName: currentTeamName,
    };
  }

  const [players, program] = await Promise.all([
    readTeam(currentPlayerRepository(), currentMembershipRepository(), {
      seasonStartingYear: currentSeasonStartingYear,
      teamName: currentTeamName,
    }),
    loadProgram(),
  ]);

  return {
    events: program.events,
    players,
    selectedEventId,
    season: { endingYear: currentSeasonStartingYear + 1, startingYear: currentSeasonStartingYear },
    teamName: currentTeamName,
  };
};
