import { eq } from 'drizzle-orm';
import { afterAll, beforeAll, describe, expect, it } from 'vitest';

import type { DutyRepository } from '../../application/duties/duty-repository';
import { createDatabase } from './database';
import { createPostgresDutyRepository } from './postgres-duty-repository';
import { gameFixtures, gameOccurrences, locations, players, teams } from './schema';

const databaseUrl = process.env.DATABASE_URL;

if (!databaseUrl) {
  describe.skip('PostgreSQL duty repository', () => {});
} else {
  describe('PostgreSQL duty repository', () => {
    const playerAssociationIds = ['duty-integration-avery', 'duty-integration-blake'];
    const locationName = 'Duty integration court';
    const homeTeamName = 'Duty integration home team';
    const awayTeamName = 'Duty integration away team';
    const database = createDatabase(databaseUrl);
    const repository: DutyRepository = createPostgresDutyRepository(database);

    let playerIds: string[];
    let occurrenceId: string;

    beforeAll(async () => {
      await database.delete(players).where(eq(players.associationId, playerAssociationIds[0]));
      await database.delete(players).where(eq(players.associationId, playerAssociationIds[1]));
      await database.delete(teams).where(eq(teams.name, homeTeamName));
      await database.delete(teams).where(eq(teams.name, awayTeamName));

      const insertedPlayers = await database
        .insert(players)
        .values(
          playerAssociationIds.map((associationId, index) => ({
            associationId,
            birthDate: `201${index + 1}-01-01`,
            firstName: `Duty integration ${index}`,
          })),
        )
        .returning({ associationId: players.associationId, id: players.id });

      playerIds = playerAssociationIds.map((associationId) => {
        const player = insertedPlayers.find((row) => row.associationId === associationId);

        if (!player) {
          throw new Error(`Could not create duty integration fixture ${associationId}`);
        }

        return player.id;
      });

      const [location] = await database
        .insert(locations)
        .values({ name: locationName, travelMinutes: 0 })
        .returning({ id: locations.id });
      const [homeTeam] = await database
        .insert(teams)
        .values({ isOwnTeam: true, name: homeTeamName })
        .returning({ id: teams.id });
      const [awayTeam] = await database
        .insert(teams)
        .values({ isOwnTeam: false, name: awayTeamName })
        .returning({ id: teams.id });

      if (!location || !homeTeam || !awayTeam) {
        throw new Error('Could not create duty integration location/teams');
      }

      const [fixture] = await database
        .insert(gameFixtures)
        .values({ awayTeamId: awayTeam.id, homeTeamId: homeTeam.id })
        .returning({ id: gameFixtures.id });

      if (!fixture) {
        throw new Error('Could not create duty integration fixture');
      }

      const [occurrence] = await database
        .insert(gameOccurrences)
        .values({
          arrivalBufferMinutes: 30,
          date: '2026-08-15',
          fixtureId: fixture.id,
          locationId: location.id,
          startTime: '14:30',
          status: 'scheduled',
          travelMinutes: 20,
        })
        .returning({ id: gameOccurrences.id });

      if (!occurrence) {
        throw new Error('Could not create duty integration occurrence');
      }

      occurrenceId = occurrence.id;
    });

    afterAll(async () => {
      await database.delete(gameOccurrences).where(eq(gameOccurrences.id, occurrenceId));
      await database.delete(locations).where(eq(locations.name, locationName));
      await database.delete(teams).where(eq(teams.name, homeTeamName));
      await database.delete(teams).where(eq(teams.name, awayTeamName));
      await database.delete(players).where(eq(players.associationId, playerAssociationIds[0]));
      await database.delete(players).where(eq(players.associationId, playerAssociationIds[1]));
      await database.close();
    });

    it('records a signup for an existing player and rejects a nonexistent one', async () => {
      const view = await repository.recordSignup({
        dutyType: 'referee',
        occurrenceId,
        playerId: playerIds[0],
        status: 'volunteer',
      });

      expect(view.signups).toEqual(
        expect.arrayContaining([
          expect.objectContaining({ playerId: playerIds[0], status: 'volunteer' }),
        ]),
      );

      await expect(
        repository.recordSignup({
          dutyType: 'referee',
          occurrenceId,
          playerId: '00000000-0000-0000-0000-000000000000',
          status: 'volunteer',
        }),
      ).rejects.toThrow('00000000-0000-0000-0000-000000000000 does not exist');
    });

    it('assigns a slot to an existing player and rejects a nonexistent one', async () => {
      const configured = await repository.configure(occurrenceId, {
        drivingSlots: 0,
        jurySlots: 0,
        refereeSlots: 1,
      });
      const slot = configured.slots.find((candidate) => candidate.dutyType === 'referee');

      if (!slot) {
        throw new Error('Could not create duty integration slot');
      }

      const assigned = await repository.assign(slot.id, playerIds[1]);

      expect(assigned.slots).toEqual(
        expect.arrayContaining([
          expect.objectContaining({ assignedPlayerId: playerIds[1], id: slot.id }),
        ]),
      );

      await expect(
        repository.assign(slot.id, '00000000-0000-0000-0000-000000000000'),
      ).rejects.toThrow('00000000-0000-0000-0000-000000000000 does not exist');
    });
  });
}
