import { json } from '@sveltejs/kit';

import { configureTeam } from '$lib/application/teams/configure-team';
import { currentTeamRepository } from '$lib/server/composition-root';

const readName = async (request: Request): Promise<string | undefined> => {
  try {
    const payload: unknown = await request.json();

    if (typeof payload === 'object' && payload !== null && 'name' in payload) {
      const { name } = payload;

      if (typeof name === 'string' && name.trim()) {
        return name.trim();
      }
    }
  } catch {
    // The endpoint reports all malformed bodies as an invalid team name.
  }

  return undefined;
};

export const POST = async ({ request }) => {
  const name = await readName(request);

  if (!name) {
    return json({ error: 'invalid_team_name' }, { status: 400 });
  }

  const team = await configureTeam(currentTeamRepository(), name);

  return json(team);
};
