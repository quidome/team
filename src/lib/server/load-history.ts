import { env } from '$env/dynamic/private';

import { readHistory } from '$lib/application/history/read-history';
import {
  currentDutyRepository,
  currentParticipationRepository,
} from '$lib/server/composition-root';
import { currentTeamName, loadTeam } from './load-team';

export const loadHistory = async () => {
  if (!env.DATABASE_URL?.trim()) {
    return { entries: [], players: [] };
  }

  const team = await loadTeam();
  const [records, fairness] = await Promise.all([
    currentParticipationRepository().findAll(),
    currentDutyRepository().findFairness(),
  ]);

  return readHistory(team.players, records, fairness, team.events, currentTeamName);
};
