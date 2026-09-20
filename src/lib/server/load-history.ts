import { env } from '$env/dynamic/private';

import { readHistory } from '$lib/application/history/read-history';
import {
  currentAuditRepository,
  currentDutyRepository,
  currentParticipationRepository,
} from '$lib/server/composition-root';
import { currentTeamName, loadTeam } from './load-team';

export const loadHistory = async () => {
  if (!env.DATABASE_URL?.trim()) {
    return { auditEntries: [], entries: [], players: [] };
  }

  const team = await loadTeam();
  const [records, fairness, auditEntries] = await Promise.all([
    currentParticipationRepository().findAll(),
    currentDutyRepository().findFairness(),
    currentAuditRepository().findAll(),
  ]);

  return {
    ...readHistory(team.players, records, fairness, team.events, currentTeamName),
    auditEntries,
  };
};
