import type { TeamRepository } from '../teams/team-repository';
import type { Coordinator, CoordinatorRepository } from './coordinator-repository';

export class CoordinatorTeamError extends Error {}

export const recordCoordinatorLogin = (
  coordinators: CoordinatorRepository,
  subject: string,
): Promise<Coordinator> => coordinators.ensure(subject);

export const takeTeamResponsibility = async (
  coordinators: CoordinatorRepository,
  teams: TeamRepository,
  subject: string,
  teamName: string,
): Promise<Coordinator> => {
  const team = await teams.findByName(teamName);

  if (!team) {
    throw new CoordinatorTeamError(`Team ${teamName} does not exist`);
  }

  const coordinator = await coordinators.ensure(subject);

  if (coordinator.teamNames.includes(teamName)) {
    return coordinator;
  }

  return coordinators.addTeam(subject, teamName);
};

export const giveUpTeamResponsibility = async (
  coordinators: CoordinatorRepository,
  subject: string,
  teamName: string,
): Promise<Coordinator> => {
  const coordinator = await coordinators.ensure(subject);

  if (!coordinator.teamNames.includes(teamName)) {
    return coordinator;
  }

  if (coordinator.defaultTeamName === teamName) {
    await coordinators.setDefaultTeam(subject, undefined);
  }

  return coordinators.removeTeam(subject, teamName);
};

export const setDefaultTeam = async (
  coordinators: CoordinatorRepository,
  subject: string,
  teamName: string,
): Promise<Coordinator> => {
  const coordinator = await coordinators.ensure(subject);

  if (!coordinator.teamNames.includes(teamName)) {
    throw new CoordinatorTeamError(`Coordinator is not responsible for ${teamName}`);
  }

  return coordinators.setDefaultTeam(subject, teamName);
};
