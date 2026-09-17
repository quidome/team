import type { Team, TeamRepository } from './team-repository';

export const configureTeam = async (teams: TeamRepository, name: string): Promise<Team> => {
  const existingTeam = await teams.findByName(name);

  if (existingTeam) {
    return existingTeam;
  }

  return teams.save({ name });
};
