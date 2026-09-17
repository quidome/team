import { eq } from 'drizzle-orm';
import { afterAll, beforeAll, describe, expect, it } from 'vitest';

import type { PlayerRepository } from '../../application/players/player-repository';
import { createDatabase } from './database';
import { createPostgresPlayerRepository } from './postgres-player-repository';
import { players } from './schema';

const databaseUrl = process.env.DATABASE_URL;

if (!databaseUrl) {
  describe.skip('PostgreSQL player repository', () => {});
} else {
  describe('PostgreSQL player repository', () => {
    const player = {
      associationId: 'integration-test-player',
      birthDate: '2011-06-15',
      name: 'Avery',
    };
    const database = createDatabase(databaseUrl);
    const repository: PlayerRepository = createPostgresPlayerRepository(database);

    beforeAll(async () => {
      await database.delete(players).where(eq(players.associationId, player.associationId));
    });

    afterAll(async () => {
      await database.delete(players).where(eq(players.associationId, player.associationId));
      await database.close();
    });

    it('persists and retrieves a registered player', async () => {
      await expect(repository.save(player)).resolves.toEqual(player);
      await expect(repository.findByAssociationId(player.associationId)).resolves.toEqual(player);
    });
  });
}
