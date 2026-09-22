import { env } from '$env/dynamic/private';
import { dev } from '$app/environment';

import { formatTeamSeasonContext } from '$lib/application/team-context';
import { currentTeamSeasonContext } from '$lib/server/load-team';

export const load = async ({ locals }) => ({
  isAuthenticated: Boolean(locals.coordinatorSession),
  isDeveloperMode: dev && env.DEV_AUTH_BYPASS?.trim().toLowerCase() === 'true',
  teamSeasonLabel: formatTeamSeasonContext(await currentTeamSeasonContext()),
});
