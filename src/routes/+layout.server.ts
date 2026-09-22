import { formatTeamSeasonContext } from '$lib/application/team-context';
import { currentTeamSeasonContext } from '$lib/server/load-team';

export const load = async ({ locals }) => ({
  isAuthenticated: Boolean(locals.coordinatorSession),
  teamSeasonLabel: formatTeamSeasonContext(await currentTeamSeasonContext()),
});
