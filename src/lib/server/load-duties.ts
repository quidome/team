import type { ProgramGameEvent } from '$lib/application/program/read-program';
import { loadTeam, type TeamPageData } from '$lib/server/load-team';

export interface DutiesPageData {
  events: ProgramGameEvent[];
  players: TeamPageData['players'];
  season: TeamPageData['season'];
  teamName: string;
}

export const loadDuties = async (): Promise<DutiesPageData> => {
  const team = await loadTeam();
  const events = team.events.filter((event): event is ProgramGameEvent => event.type === 'game');

  return { ...team, events };
};
