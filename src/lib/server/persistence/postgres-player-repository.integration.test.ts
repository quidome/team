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
      firstName: 'Avery',
      lastName: 'Smith',
    };
    const minimalPlayerName = 'Integration Minimal Player';
    const database = createDatabase(databaseUrl);
    const repository: PlayerRepository = createPostgresPlayerRepository(database);

    beforeAll(async () => {
      await database.delete(players).where(eq(players.associationId, player.associationId));
      await database.delete(players).where(eq(players.firstName, minimalPlayerName));
    });

    afterAll(async () => {
      await database.delete(players).where(eq(players.associationId, player.associationId));
      await database.delete(players).where(eq(players.firstName, minimalPlayerName));
      await database.close();
    });

    it('persists, retrieves, and updates a registered player', async () => {
      const stored = await repository.save(player);

      expect(stored).toEqual({ id: expect.any(String), ...player });
      await expect(repository.findByAssociationId(player.associationId)).resolves.toEqual(stored);
      await expect(repository.findById(stored.id)).resolves.toEqual(stored);

      const updated = await repository.update({ ...stored, lastName: 'Jones' });

      expect(updated).toEqual({ ...stored, lastName: 'Jones' });
      await expect(repository.findById(stored.id)).resolves.toEqual(updated);

      await repository.deleteById(stored.id);
      await expect(repository.findById(stored.id)).resolves.toBeUndefined();
    });

    it('persists a player with only a first name', async () => {
      const stored = await repository.save({ firstName: minimalPlayerName });

      expect(stored).toEqual({ id: expect.any(String), firstName: minimalPlayerName });
      await expect(repository.findById(stored.id)).resolves.toEqual(stored);
    });
  });
}
