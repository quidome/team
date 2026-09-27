import { json } from '@sveltejs/kit';

import {
  CoordinatorTeamError,
  giveUpTeamResponsibility,
  setDefaultTeam,
  takeTeamResponsibility,
} from '$lib/application/coordinators/coordinator-teams';
import {
  currentAuditRepository,
  currentCoordinatorRepository,
  currentTeamRepository,
} from '$lib/server/composition-root';

const readTeamName = async (request: Request): Promise<string | undefined> => {
  try {
    const payload: unknown = await request.json();

    if (typeof payload === 'object' && payload !== null && 'teamName' in payload) {
      const { teamName } = payload;

      if (typeof teamName === 'string' && teamName.trim()) {
        return teamName.trim();
      }
    }
  } catch {
    // The endpoint reports all malformed bodies as an invalid team name.
  }

  return undefined;
};

const sessionSubject = (locals: App.Locals): string | undefined =>
  locals.coordinatorSession?.subject;

const unauthenticated = () => json({ error: 'coordinator_session_required' }, { status: 401 });

const invalidTeamName = () => json({ error: 'invalid_team_name' }, { status: 400 });

const rejectCoordinatorTeamErrors = (error: unknown) => {
  if (error instanceof CoordinatorTeamError) {
    return json({ error: error.message }, { status: 400 });
  }

  throw error;
};

export const GET = async ({ locals }) => {
  const subject = sessionSubject(locals);

  if (!subject) {
    return unauthenticated();
  }

  return json(await currentCoordinatorRepository().ensure(subject));
};

/** Takes responsibility for a team. */
export const POST = async ({ locals, request }) => {
  const subject = sessionSubject(locals);

  if (!subject) {
    return unauthenticated();
  }

  const teamName = await readTeamName(request);

  if (!teamName) {
    return invalidTeamName();
  }

  try {
    const coordinator = await takeTeamResponsibility(
      currentCoordinatorRepository(),
      currentTeamRepository(),
      subject,
      teamName,
    );

    await currentAuditRepository().record({
      action: 'coordinator_team_taken',
      entityId: subject,
      entityType: 'coordinator',
      metadata: { teamName },
    });

    return json(coordinator);
  } catch (error) {
    return rejectCoordinatorTeamErrors(error);
  }
};

/** Sets the default team. */
export const PUT = async ({ locals, request }) => {
  const subject = sessionSubject(locals);

  if (!subject) {
    return unauthenticated();
  }

  const teamName = await readTeamName(request);

  if (!teamName) {
    return invalidTeamName();
  }

  try {
    const coordinator = await setDefaultTeam(currentCoordinatorRepository(), subject, teamName);

    await currentAuditRepository().record({
      action: 'coordinator_default_team_set',
      entityId: subject,
      entityType: 'coordinator',
      metadata: { teamName },
    });

    return json(coordinator);
  } catch (error) {
    return rejectCoordinatorTeamErrors(error);
  }
};

/** Gives up responsibility for a team. */
export const DELETE = async ({ locals, url }) => {
  const subject = sessionSubject(locals);

  if (!subject) {
    return unauthenticated();
  }

  const teamName = url.searchParams.get('teamName')?.trim();

  if (!teamName) {
    return invalidTeamName();
  }

  const coordinator = await giveUpTeamResponsibility(
    currentCoordinatorRepository(),
    subject,
    teamName,
  );

  await currentAuditRepository().record({
    action: 'coordinator_team_given_up',
    entityId: subject,
    entityType: 'coordinator',
    metadata: { teamName },
  });

  return json(coordinator);
};
