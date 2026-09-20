import type { DutyView } from '$lib/application/duties/duty-repository';
import type { ProgramGameEvent } from '$lib/application/program/read-program';
import { completeDueDuties } from '$lib/application/duties/manage-duties';
import { currentDutyRepository, currentGameRepository } from '$lib/server/composition-root';
import { loadTeam, type TeamPageData } from '$lib/server/load-team';

export interface DutiesPageData {
  duties: Record<string, DutyView>;
  events: ProgramGameEvent[];
  players: TeamPageData['players'];
  season: TeamPageData['season'];
  teamName: string;
}

export const loadDuties = async (): Promise<DutiesPageData> => {
  const team = await loadTeam();
  const events = team.events.filter(
    (event): event is ProgramGameEvent =>
      event.type === 'game' &&
      (event.homeTeamName === team.teamName || event.awayTeamName === team.teamName),
  );

  if (events.length === 0) {
    return { duties: {}, ...team, events };
  }

  const duties = currentDutyRepository();
  const games = currentGameRepository();
  await completeDueDuties(duties, games);
  const views = await Promise.all(
    events.map(async (event) => [event.id, await duties.findByOccurrence(event.id)] as const),
  );

  return { duties: Object.fromEntries(views), ...team, events };
};
