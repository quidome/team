import type { TeamRepository } from '../teams/team-repository';
import type {
  Coordinator,
  CoordinatorProfile,
  CoordinatorRepository,
} from './coordinator-repository';

export class CoordinatorError extends Error {}

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
    throw new CoordinatorError(`Team ${teamName} does not exist`);
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
    throw new CoordinatorError(`Coordinator is not responsible for ${teamName}`);
  }

  return coordinators.setDefaultTeam(subject, teamName);
};

const emailPattern = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;

export const setCoordinatorProfile = async (
  coordinators: CoordinatorRepository,
  subject: string,
  input: { displayName: string; email?: string },
): Promise<Coordinator> => {
  const displayName = input.displayName.trim();
  const email = input.email?.trim();

  if (!displayName) {
    throw new CoordinatorError('Display name is required');
  }

  if (email && !emailPattern.test(email)) {
    throw new CoordinatorError('Email address is not valid');
  }

  const profile: CoordinatorProfile = email ? { displayName, email } : { displayName };

  await coordinators.ensure(subject);

  return coordinators.setProfile(subject, profile);
};
