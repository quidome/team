import { asc, eq } from 'drizzle-orm';

import type { Player, PlayerRepository } from '../../application/players/player-repository';
import type { DatabaseConnection } from './database';
import { players } from './schema';

const columns = {
  id: players.id,
  associationId: players.associationId,
  birthDate: players.birthDate,
  firstName: players.firstName,
  lastName: players.lastName,
};

const toPlayer = (row: {
  id: string;
  associationId: string | null;
  birthDate: string | null;
  firstName: string;
  lastName: string | null;
}): Player => ({
  id: row.id,
  ...(row.associationId ? { associationId: row.associationId } : {}),
  ...(row.birthDate ? { birthDate: row.birthDate } : {}),
  firstName: row.firstName,
  ...(row.lastName ? { lastName: row.lastName } : {}),
});

export const createPostgresPlayerRepository = (database: DatabaseConnection): PlayerRepository => ({
  async findAll(): Promise<Player[]> {
    const rows = await database.select(columns).from(players).orderBy(asc(players.firstName));

    return rows.map(toPlayer);
  },

  async findById(id: string): Promise<Player | undefined> {
    const [row] = await database.select(columns).from(players).where(eq(players.id, id)).limit(1);

    return row ? toPlayer(row) : undefined;
  },

  async findByAssociationId(associationId: string): Promise<Player | undefined> {
    const [row] = await database
      .select(columns)
      .from(players)
      .where(eq(players.associationId, associationId))
      .limit(1);

    return row ? toPlayer(row) : undefined;
  },

  async save(player: Omit<Player, 'id'>): Promise<Player> {
    const [row] = await database
      .insert(players)
      .values({
        associationId: player.associationId ?? null,
        birthDate: player.birthDate ?? null,
        firstName: player.firstName,
        lastName: player.lastName ?? null,
      })
      .returning(columns);

    if (!row) {
      throw new Error('PostgreSQL did not return the stored player');
    }

    return toPlayer(row);
  },

  async update(player: Player): Promise<Player> {
    const [row] = await database
      .update(players)
      .set({
        associationId: player.associationId ?? null,
        birthDate: player.birthDate ?? null,
        firstName: player.firstName,
        lastName: player.lastName ?? null,
      })
      .where(eq(players.id, player.id))
      .returning(columns);

    if (!row) {
      throw new Error('Player does not exist');
    }

    return toPlayer(row);
  },

  async deleteById(id: string): Promise<void> {
    await database.delete(players).where(eq(players.id, id));
  },
});
