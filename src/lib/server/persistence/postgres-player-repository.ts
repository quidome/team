import { eq } from 'drizzle-orm';

import type { Player, PlayerRepository } from '../../application/players/player-repository';
import { createDatabase } from './database';
import { players } from './schema';

type Database = ReturnType<typeof createDatabase>;

export const createPostgresPlayerRepository = (database: Database): PlayerRepository => ({
  async findByAssociationId(associationId: string): Promise<Player | undefined> {
    const [player] = await database
      .select({
        associationId: players.associationId,
        birthDate: players.birthDate,
        name: players.name,
      })
      .from(players)
      .where(eq(players.associationId, associationId))
      .limit(1);

    return player;
  },

  async save(player: Player): Promise<Player> {
    const [storedPlayer] = await database.insert(players).values(player).returning({
      associationId: players.associationId,
      birthDate: players.birthDate,
      name: players.name,
    });

    if (!storedPlayer) {
      throw new Error('PostgreSQL did not return the stored player');
    }

    return storedPlayer;
  },
});
