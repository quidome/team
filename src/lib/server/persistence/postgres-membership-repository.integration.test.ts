import { and, eq } from 'drizzle-orm';
import { afterAll, beforeAll, describe, expect, it } from 'vitest';

import type { MembershipRepository } from '../../application/memberships/membership-repository';
import { createDatabase } from './database';
import { createPostgresMembershipRepository } from './postgres-membership-repository';
import { memberships, players, seasons, teams } from './schema';

const databaseUrl = process.env.DATABASE_URL;

if (!databaseUrl) {
  describe.skip('PostgreSQL membership repository', () => {});
} else {
  describe('PostgreSQL membership repository', () => {
    const membership = {
      jerseyNumber: 7,
      participationType: 'trains_and_plays' as const,
      playerAssociationId: 'integration-membership-player',
      relationship: 'primary' as const,
      seasonStartingYear: 2097,
      status: 'active' as const,
      teamName: 'U16-1 membership integration',
    };
    const database = createDatabase(databaseUrl);
    const repository: MembershipRepository = createPostgresMembershipRepository(database);

    let playerId: string;
    let seasonId: string;
    let teamId: string;

    beforeAll(async () => {
      const [existingPlayer] = await database
        .select({ id: players.id })
        .from(players)
        .where(eq(players.associationId, membership.playerAssociationId));
      const [existingSeason] = await database
        .select({ id: seasons.id })
        .from(seasons)
        .where(eq(seasons.startingYear, membership.seasonStartingYear));
      const [existingTeam] = await database
        .select({ id: teams.id })
        .from(teams)
        .where(eq(teams.name, membership.teamName));

      if (existingPlayer && existingSeason && existingTeam) {
        await database
          .delete(memberships)
          .where(
            and(
              eq(memberships.playerId, existingPlayer.id),
              eq(memberships.seasonId, existingSeason.id),
              eq(memberships.teamId, existingTeam.id),
            ),
          );
      }

      if (existingPlayer) {
        await database.delete(players).where(eq(players.id, existingPlayer.id));
      }
      if (existingSeason) {
        await database.delete(seasons).where(eq(seasons.id, existingSeason.id));
      }
      if (existingTeam) {
        await database.delete(teams).where(eq(teams.id, existingTeam.id));
      }

      const [player] = await database
        .insert(players)
        .values({
          associationId: membership.playerAssociationId,
          birthDate: '2011-06-15',
          name: 'Avery',
        })
        .returning({ id: players.id });
      const [season] = await database
        .insert(seasons)
        .values({ endingYear: 2098, startingYear: membership.seasonStartingYear })
        .returning({ id: seasons.id });
      const [team] = await database
        .insert(teams)
        .values({ name: membership.teamName })
        .returning({ id: teams.id });

      if (!player || !season || !team) {
        throw new Error('Could not create membership integration fixtures');
      }

      playerId = player.id;
      seasonId = season.id;
      teamId = team.id;
    });

    afterAll(async () => {
      await database
        .delete(memberships)
        .where(
          and(
            eq(memberships.playerId, playerId),
            eq(memberships.seasonId, seasonId),
            eq(memberships.teamId, teamId),
          ),
        );
      await database.delete(players).where(eq(players.id, playerId));
      await database.delete(seasons).where(eq(seasons.id, seasonId));
      await database.delete(teams).where(eq(teams.id, teamId));
      await database.close();
    });

    it('persists and retrieves a player membership', async () => {
      await expect(repository.save(membership)).resolves.toEqual(membership);
      await expect(repository.find(membership)).resolves.toEqual(membership);
    });
  });
}
