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
        playerId: memberships.playerId,
        relationship: memberships.relationship,
        seasonStartingYear: seasons.startingYear,
        status: memberships.status,
        teamName: teams.name,
      })
      .from(memberships)
      .innerJoin(players, eq(memberships.playerId, players.id))
      .innerJoin(teams, eq(memberships.teamId, teams.id))
      .innerJoin(seasons, eq(memberships.seasonId, seasons.id))
      .orderBy(asc(players.firstName));

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
        playerId: memberships.playerId,
        relationship: memberships.relationship,
        seasonStartingYear: seasons.startingYear,
        status: memberships.status,
        teamName: teams.name,
      })
      .from(memberships)
      .innerJoin(teams, eq(memberships.teamId, teams.id))
      .innerJoin(seasons, eq(memberships.seasonId, seasons.id))
      .where(
        and(
          eq(memberships.playerId, membership.playerId),
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
      .where(eq(players.id, membership.playerId))
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
      throw new Error(`Player ${membership.playerId} does not exist`);
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

  async update(membership: Membership): Promise<Membership> {
    const [player] = await database
      .select({ id: players.id })
      .from(players)
      .where(eq(players.id, membership.playerId))
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

    if (!player || !season || !team) {
      throw new Error('Membership context does not exist');
    }

    const [updatedMembership] = await database
      .update(memberships)
      .set({
        jerseyNumber: membership.jerseyNumber,
        participationType: membership.participationType,
        relationship: membership.relationship,
        status: membership.status,
      })
      .where(
        and(
          eq(memberships.playerId, player.id),
          eq(memberships.seasonId, season.id),
          eq(memberships.teamId, team.id),
        ),
      )
      .returning({ id: memberships.id });

    if (!updatedMembership) {
      throw new Error('Membership does not exist');
    }

    return membership;
  },
});
