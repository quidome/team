import { eq } from 'drizzle-orm';
import { afterAll, beforeAll, describe, expect, it } from 'vitest';

import type { ParticipationRepository } from '../../application/participation/participation-repository';
import { createDatabase } from './database';
import { createPostgresParticipationRepository } from './postgres-participation-repository';
import { players } from './schema';

const databaseUrl = process.env.DATABASE_URL;

if (!databaseUrl) {
  describe.skip('PostgreSQL participation repository', () => {});
} else {
  describe('PostgreSQL participation repository', () => {
    const playerAssociationIds = [
      'participation-integration-avery',
      'participation-integration-blake',
    ];
    const database = createDatabase(databaseUrl);
    const repository: ParticipationRepository = createPostgresParticipationRepository(database);

    let playerIds: string[];

    beforeAll(async () => {
      await database.delete(players).where(eq(players.associationId, playerAssociationIds[0]));
      await database.delete(players).where(eq(players.associationId, playerAssociationIds[1]));

      const insertedPlayers = await database
        .insert(players)
        .values(
          playerAssociationIds.map((associationId, index) => ({
            associationId,
            birthDate: `201${index + 1}-01-01`,
            firstName: `Participation integration ${index}`,
          })),
        )
        .returning({ associationId: players.associationId, id: players.id });

      playerIds = playerAssociationIds.map((associationId) => {
        const player = insertedPlayers.find((row) => row.associationId === associationId);

        if (!player) {
          throw new Error(`Could not create participation integration fixture ${associationId}`);
        }

        return player.id;
      });
    });

    afterAll(async () => {
      await database.delete(players).where(eq(players.associationId, playerAssociationIds[0]));
      await database.delete(players).where(eq(players.associationId, playerAssociationIds[1]));
      await database.close();
    });

    it('persists and updates occurrence participation', async () => {
      const initialRecords = await repository.saveMany([
        {
          occurrenceId: '00000000-0000-0000-0000-000000000001',
          occurrenceType: 'training',
          playerId: playerIds[0],
          status: 'present',
        },
        {
          absenceReason: 'illness',
          occurrenceId: '00000000-0000-0000-0000-000000000001',
          occurrenceType: 'training',
          playerId: playerIds[1],
          status: 'absent',
        },
      ]);

      expect(initialRecords).toEqual([
        expect.objectContaining({
          playerId: playerIds[0],
          status: 'present',
        }),
        expect.objectContaining({
          absenceReason: 'illness',
          playerId: playerIds[1],
          status: 'absent',
        }),
      ]);

      await repository.saveMany([
        {
          occurrenceId: '00000000-0000-0000-0000-000000000001',
          occurrenceType: 'training',
          playerId: playerIds[1],
          status: 'present',
        },
      ]);

      await expect(
        repository.findByOccurrence('training', '00000000-0000-0000-0000-000000000001'),
      ).resolves.toEqual(
        expect.arrayContaining([
          expect.objectContaining({
            playerId: playerIds[0],
            status: 'present',
          }),
          expect.objectContaining({
            playerId: playerIds[1],
            status: 'present',
          }),
        ]),
      );
    });
  });
}
