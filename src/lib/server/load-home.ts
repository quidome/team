import { env } from '$env/dynamic/private';

import { readTasks } from '$lib/application/tasks/generate-tasks';
import type { Task } from '$lib/application/tasks/task-repository';
import type { TeamPlayer } from '$lib/application/team/read-team';
import { currentTaskRepository } from '$lib/server/composition-root';
import { loadTeam } from '$lib/server/load-team';
import type { ProgramEvent } from '$lib/application/program/read-program';

export interface HomePageData {
  events: ProgramEvent[];
  players: TeamPlayer[];
  seasonStartingYear: number;
  tasks: Task[];
  teamName: string;
}

const today = () => new Intl.DateTimeFormat('en-CA', { timeZone: 'UTC' }).format(new Date());

export const loadHome = async (): Promise<HomePageData> => {
  const team = await loadTeam();

  if (!env.DATABASE_URL?.trim()) {
    return {
      events: team.events,
      players: team.players,
      seasonStartingYear: team.season.startingYear,
      tasks: [],
      teamName: team.teamName,
    };
  }

  return {
    events: team.events,
    players: team.players,
    seasonStartingYear: team.season.startingYear,
    tasks: await readTasks(currentTaskRepository(), team.events, {
      teamName: team.teamName,
      today: today(),
    }),
    teamName: team.teamName,
  };
};
