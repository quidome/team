import { calculateNormalAgeGroup } from '../../domain/normal-age-group';
import type { Membership, MembershipRepository } from '../memberships/membership-repository';
import type { Player, PlayerRepository } from '../players/player-repository';

export interface TeamPlayer extends Player {
  membership?: Membership;
  normalAgeGroup: ReturnType<typeof calculateNormalAgeGroup>;
}

export interface TeamRosterOptions {
  seasonStartingYear: number;
  teamName: string;
}

export const readTeam = async (
  players: PlayerRepository,
  memberships: MembershipRepository,
  options: TeamRosterOptions,
): Promise<TeamPlayer[]> => {
  const [storedPlayers, storedMemberships] = await Promise.all([
    players.findAll(),
    memberships.findAll(),
  ]);
  const membershipsByPlayer = new Map(
    storedMemberships
      .filter(
        (membership) =>
          membership.seasonStartingYear === options.seasonStartingYear &&
          membership.teamName === options.teamName,
      )
      .map((membership) => [membership.playerAssociationId, membership]),
  );

  return storedPlayers.map((player) => ({
    ...player,
    ...(membershipsByPlayer.has(player.associationId)
      ? { membership: membershipsByPlayer.get(player.associationId) }
      : {}),
    normalAgeGroup: calculateNormalAgeGroup(
      options.seasonStartingYear,
      Number(player.birthDate.slice(0, 4)),
    ),
  }));
};
