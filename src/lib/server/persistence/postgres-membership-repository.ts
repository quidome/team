import { and, asc, eq } from 'drizzle-orm';

import type {
  Membership,
  MembershipRepository,
} from '../../application/memberships/membership-repository';
import { createDatabase } from './database';
import { memberships, players, seasons, teams } from './schema';

type Database = ReturnType<typeof createDatabase>;

export const createPostgresMembershipRepository = (database: Database): MembershipRepository => ({
  async findAll(): Promise<Membership[]> {
    const storedMemberships = await database
      .select({
        jerseyNumber: memberships.jerseyNumber,
        participationType: memberships.participationType,
        playerAssociationId: players.associationId,
        relationship: memberships.relationship,
        seasonStartingYear: seasons.startingYear,
        status: memberships.status,
        teamName: teams.name,
      })
      .from(memberships)
      .innerJoin(players, eq(memberships.playerId, players.id))
      .innerJoin(teams, eq(memberships.teamId, teams.id))
      .innerJoin(seasons, eq(memberships.seasonId, seasons.id))
      .orderBy(asc(players.name));

    return storedMemberships.map((storedMembership) => ({
      ...storedMembership,
      ...(storedMembership.jerseyNumber === null
        ? { jerseyNumber: undefined }
        : { jerseyNumber: storedMembership.jerseyNumber }),
    }));
  },

  async find(membership: Membership): Promise<Membership | undefined> {
    const [storedMembership] = await database
      .select({
        jerseyNumber: memberships.jerseyNumber,
        participationType: memberships.participationType,
        playerAssociationId: players.associationId,
        relationship: memberships.relationship,
        seasonStartingYear: seasons.startingYear,
        status: memberships.status,
        teamName: teams.name,
      })
      .from(memberships)
      .innerJoin(players, eq(memberships.playerId, players.id))
      .innerJoin(teams, eq(memberships.teamId, teams.id))
      .innerJoin(seasons, eq(memberships.seasonId, seasons.id))
      .where(
        and(
          eq(players.associationId, membership.playerAssociationId),
          eq(seasons.startingYear, membership.seasonStartingYear),
          eq(teams.name, membership.teamName),
        ),
      )
      .limit(1);

    if (!storedMembership) {
      return undefined;
    }

    return {
      ...storedMembership,
      ...(storedMembership.jerseyNumber === null
        ? { jerseyNumber: undefined }
        : { jerseyNumber: storedMembership.jerseyNumber }),
    };
  },

  async save(membership: Membership): Promise<Membership> {
    const [player] = await database
      .select({ id: players.id })
      .from(players)
      .where(eq(players.associationId, membership.playerAssociationId))
      .limit(1);
    const [season] = await database
      .select({ id: seasons.id })
      .from(seasons)
      .where(eq(seasons.startingYear, membership.seasonStartingYear))
      .limit(1);
    const [team] = await database
      .select({ id: teams.id })
      .from(teams)
      .where(eq(teams.name, membership.teamName))
      .limit(1);

    if (!player) {
      throw new Error(`Player ${membership.playerAssociationId} does not exist`);
    }

    if (!season) {
      throw new Error(`Season ${membership.seasonStartingYear} does not exist`);
    }

    if (!team) {
      throw new Error(`Team ${membership.teamName} does not exist`);
    }

    await database.insert(memberships).values({
      jerseyNumber: membership.jerseyNumber,
      participationType: membership.participationType,
      playerId: player.id,
      relationship: membership.relationship,
      seasonId: season.id,
      status: membership.status,
      teamId: team.id,
    });

    return membership;
  },
});
