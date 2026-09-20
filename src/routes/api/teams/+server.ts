import { json } from '@sveltejs/kit';

import { configureTeam } from '$lib/application/teams/configure-team';
import { currentAuditRepository, currentTeamRepository } from '$lib/server/composition-root';

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

const readTeamUpdate = async (
  request: Request,
): Promise<{ currentName: string; name: string } | undefined> => {
  try {
    const payload: unknown = await request.json();

    if (typeof payload === 'object' && payload !== null) {
      const { currentName, name } = payload as Record<string, unknown>;

      if (
        typeof currentName === 'string' &&
        currentName.trim() &&
        typeof name === 'string' &&
        name.trim()
      ) {
        return { currentName: currentName.trim(), name: name.trim() };
      }
    }
  } catch {
    // The endpoint reports malformed updates as invalid team data.
  }

  return undefined;
};

export const PUT = async ({ request }) => {
  const update = await readTeamUpdate(request);

  if (!update) {
    return json({ error: 'invalid_team_name' }, { status: 400 });
  }

  const team = await currentTeamRepository().updateName(update.currentName, update.name);
  await currentAuditRepository().record({
    action: 'team_updated',
    entityId: team.name,
    entityType: 'team',
    metadata: { name: team.name },
  });

  return json(team);
};

export const DELETE = async ({ url }) => {
  const name = url.searchParams.get('name')?.trim();

  if (!name) {
    return json({ error: 'invalid_team_name' }, { status: 400 });
  }

  await currentTeamRepository().deleteByName(name);
  await currentAuditRepository().record({
    action: 'team_deleted',
    entityId: name,
    entityType: 'team',
    metadata: { name },
  });

  return json({ deleted: true });
};

export const POST = async ({ request }) => {
  const name = await readName(request);

  if (!name) {
    return json({ error: 'invalid_team_name' }, { status: 400 });
  }

  const team = await configureTeam(currentTeamRepository(), name);

  await currentAuditRepository().record({
    action: 'team_configured',
    entityId: team.name,
    entityType: 'team',
    metadata: { name: team.name },
  });

  return json(team);
};
