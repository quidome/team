import { env } from '$env/dynamic/private';

import { readTasks } from '$lib/application/tasks/generate-tasks';
import type { Task } from '$lib/application/tasks/task-repository';
import { currentTaskRepository } from '$lib/server/composition-root';
import { currentTeamName } from '$lib/server/load-team';
import { loadProgram } from './load-program';

export interface HomePageData {
  events: Awaited<ReturnType<typeof loadProgram>>['events'];
  tasks: Task[];
  teamName: string;
}

const today = () => new Intl.DateTimeFormat('en-CA', { timeZone: 'UTC' }).format(new Date());

export const loadHome = async (): Promise<HomePageData> => {
  const program = await loadProgram();

  if (!env.DATABASE_URL?.trim()) {
    return { ...program, tasks: [], teamName: currentTeamName };
  }

  return {
    ...program,
    tasks: await readTasks(currentTaskRepository(), program.events, {
      teamName: currentTeamName,
      today: today(),
    }),
    teamName: currentTeamName,
  };
};
