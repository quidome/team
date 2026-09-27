import { json } from '@sveltejs/kit';

import {
  CoordinatorError,
  giveUpTeamResponsibility,
  setCoordinatorProfile,
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

const readProfile = async (
  request: Request,
): Promise<{ displayName: string; email?: string } | undefined> => {
  try {
    const payload: unknown = await request.json();

    if (typeof payload === 'object' && payload !== null) {
      const { displayName, email } = payload as Record<string, unknown>;

      if (typeof displayName === 'string' && (email === undefined || typeof email === 'string')) {
        return email === undefined ? { displayName } : { displayName, email };
      }
    }
  } catch {
    // The endpoint reports all malformed bodies as an invalid profile.
  }

  return undefined;
};

const sessionSubject = (locals: App.Locals): string | undefined =>
  locals.coordinatorSession?.subject;

const unauthenticated = () => json({ error: 'coordinator_session_required' }, { status: 401 });

const invalidTeamName = () => json({ error: 'invalid_team_name' }, { status: 400 });

const rejectCoordinatorErrors = (error: unknown) => {
  if (error instanceof CoordinatorError) {
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
    return rejectCoordinatorErrors(error);
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
    return rejectCoordinatorErrors(error);
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

/** Updates the coordinator's profile. */
export const PATCH = async ({ locals, request }) => {
  const subject = sessionSubject(locals);

  if (!subject) {
    return unauthenticated();
  }

  const profile = await readProfile(request);

  if (!profile) {
    return json({ error: 'invalid_coordinator_profile' }, { status: 400 });
  }

  try {
    const coordinator = await setCoordinatorProfile(
      currentCoordinatorRepository(),
      subject,
      profile,
    );

    await currentAuditRepository().record({
      action: 'coordinator_profile_updated',
      entityId: subject,
      entityType: 'coordinator',
      metadata: { displayName: coordinator.profile?.displayName ?? '' },
    });

    return json(coordinator);
  } catch (error) {
    return rejectCoordinatorErrors(error);
  }
};
