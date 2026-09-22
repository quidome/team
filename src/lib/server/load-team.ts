import { env } from '$env/dynamic/private';

import { fallbackTeamSeasonContext, type TeamSeasonContext } from '$lib/application/team-context';
import { readTeam, type TeamPlayer } from '$lib/application/team/read-team';
import { loadProgram } from '$lib/server/load-program';
import {
  currentCoordinatorSettingsRepository,
  currentMembershipRepository,
  currentPlayerRepository,
} from '$lib/server/composition-root';
import type { ProgramEvent } from '$lib/application/program/read-program';

export const currentTeamSeasonContext = async (): Promise<TeamSeasonContext> => {
  if (!env.DATABASE_URL?.trim()) {
    return fallbackTeamSeasonContext;
  }

  const settings = await currentCoordinatorSettingsRepository().get();

  return settings
    ? { seasonStartingYear: settings.seasonStartingYear, teamName: settings.primaryTeamName }
    : fallbackTeamSeasonContext;
};

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
  const context = await currentTeamSeasonContext();

  if (!env.DATABASE_URL?.trim()) {
    return {
      events: [],
      players: [],
      selectedEventId,
      season: {
        endingYear: context.seasonStartingYear + 1,
        startingYear: context.seasonStartingYear,
      },
      teamName: context.teamName,
    };
  }

  const [players, program] = await Promise.all([
    readTeam(currentPlayerRepository(), currentMembershipRepository(), {
      seasonStartingYear: context.seasonStartingYear,
      teamName: context.teamName,
    }),
    loadProgram(),
  ]);

  return {
    events: program.events,
    players,
    selectedEventId,
    season: {
      endingYear: context.seasonStartingYear + 1,
      startingYear: context.seasonStartingYear,
    },
    teamName: context.teamName,
  };
};
