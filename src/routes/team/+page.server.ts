import { loadTeam } from '$lib/server/load-team';

export const load = async ({ url }) => loadTeam(url.searchParams.get('event') ?? undefined);
