import type { TeamRepository } from '../teams/team-repository';
import type { Club, ClubRepository } from './club-repository';

export class ClubConfigurationError extends Error {}

export interface OwnClubInput {
  name: string;
  sourceClubId?: number;
}

export const configureOwnClub = async (
  clubs: ClubRepository,
  input: OwnClubInput,
): Promise<Club> => {
  const ownClub = await clubs.findOwnClub();

  if (ownClub && ownClub.name !== input.name) {
    throw new ClubConfigurationError(`Own club is already configured as ${ownClub.name}`);
  }

  if (input.sourceClubId !== undefined) {
    const sourceMatch = await clubs.findBySourceClubId(input.sourceClubId);

    if (sourceMatch && sourceMatch.name !== input.name) {
      throw new ClubConfigurationError(
        `Source club ID ${input.sourceClubId} belongs to ${sourceMatch.name}`,
      );
    }
  }

  const existing = await clubs.findByName(input.name);

  return clubs.save({
    isOwnClub: true,
    name: input.name,
    sourceClubId: input.sourceClubId ?? existing?.sourceClubId,
  });
};

export const assignTeamToClub = async (
  clubs: ClubRepository,
  teams: TeamRepository,
  clubName: string,
  teamName: string,
): Promise<void> => {
  const [club, team] = await Promise.all([clubs.findByName(clubName), teams.findByName(teamName)]);

  if (!club) {
    throw new ClubConfigurationError(`Club ${clubName} does not exist`);
  }

  if (!team) {
    throw new ClubConfigurationError(`Team ${teamName} does not exist`);
  }

  await clubs.assignTeam(clubName, teamName);
};
