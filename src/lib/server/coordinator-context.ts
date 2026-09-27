import { getRequestEvent } from '$app/server';

import { resolveCoordinatorTeamName } from '$lib/application/coordinators/coordinator-team-context';
import {
  currentCoordinatorRepository,
  currentCoordinatorSettingsRepository,
} from '$lib/server/composition-root';

const currentCoordinatorSubject = (): string | undefined => {
  try {
    return getRequestEvent().locals.coordinatorSession?.subject;
  } catch {
    // Outside a request (for example in scripts) there is no coordinator.
    return undefined;
  }
};

/** The default team of the coordinator of the current request, if any. */
export const currentCoordinatorTeamName = async (): Promise<string | undefined> => {
  const subject = currentCoordinatorSubject();

  if (!subject) {
    return (await currentCoordinatorSettingsRepository().get())?.primaryTeamName;
  }

  return resolveCoordinatorTeamName(
    currentCoordinatorRepository(),
    currentCoordinatorSettingsRepository(),
    subject,
  );
};
