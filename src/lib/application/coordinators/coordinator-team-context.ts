import type { CoordinatorSettingsRepository } from '../settings/coordinator-settings-repository';
import type { CoordinatorRepository } from './coordinator-repository';

/**
 * Returns the team a coordinator works on: their default team. A coordinator without any teams
 * takes over the installation-wide primary team from the settings, so installations that were
 * configured before coordinators existed keep their team.
 */
export const resolveCoordinatorTeamName = async (
  coordinators: CoordinatorRepository,
  settings: CoordinatorSettingsRepository,
  subject: string,
): Promise<string | undefined> => {
  const coordinator = await coordinators.ensure(subject);

  if (coordinator.defaultTeamName || coordinator.teamNames.length > 0) {
    return coordinator.defaultTeamName;
  }

  const primaryTeamName = (await settings.get())?.primaryTeamName;

  if (!primaryTeamName) {
    return undefined;
  }

  await coordinators.addTeam(subject, primaryTeamName);

  return (await coordinators.setDefaultTeam(subject, primaryTeamName)).defaultTeamName;
};
