import { eq, inArray, and } from 'drizzle-orm';
import { sql } from 'drizzle-orm';

import type {
  ParticipationOccurrenceType,
  ParticipationRecord,
  ParticipationRepository,
} from '../../application/participation/participation-repository';
import { createDatabase } from './database';
import { participationRecords, players } from './schema';

type Database = ReturnType<typeof createDatabase>;

export const createPostgresParticipationRepository = (
  database: Database,
): ParticipationRepository => ({
  async findAll(): Promise<ParticipationRecord[]> {
    const records = await database
      .select({
        absenceReason: participationRecords.absenceReason,
        occurrenceId: participationRecords.occurrenceId,
        occurrenceType: participationRecords.occurrenceType,
        playerAssociationId: players.associationId,
        status: participationRecords.status,
      })
      .from(participationRecords)
      .innerJoin(players, eq(participationRecords.playerId, players.id));

    return records.map((record) => ({
      ...(record.absenceReason === null ? {} : { absenceReason: record.absenceReason }),
      occurrenceId: record.occurrenceId,
      occurrenceType: record.occurrenceType,
      playerAssociationId: record.playerAssociationId,
      status: record.status,
    }));
  },

  async findByOccurrence(
    occurrenceType: ParticipationOccurrenceType,
    occurrenceId: string,
  ): Promise<ParticipationRecord[]> {
    const records = await database
      .select({
        absenceReason: participationRecords.absenceReason,
        occurrenceId: participationRecords.occurrenceId,
        occurrenceType: participationRecords.occurrenceType,
        playerAssociationId: players.associationId,
        status: participationRecords.status,
      })
      .from(participationRecords)
      .innerJoin(players, eq(participationRecords.playerId, players.id))
      .where(
        and(
          eq(participationRecords.occurrenceId, occurrenceId),
          eq(participationRecords.occurrenceType, occurrenceType),
        ),
      );

    return records.map((record) => ({
      ...(record.absenceReason === null ? {} : { absenceReason: record.absenceReason }),
      occurrenceId: record.occurrenceId,
      occurrenceType: record.occurrenceType,
      playerAssociationId: record.playerAssociationId,
      status: record.status,
    }));
  },

  async saveMany(records: ParticipationRecord[]): Promise<ParticipationRecord[]> {
    if (records.length === 0) {
      return [];
    }

    const playerRows = await database
      .select({ associationId: players.associationId, id: players.id })
      .from(players)
      .where(
        inArray(
          players.associationId,
          records.map((record) => record.playerAssociationId),
        ),
      );
    const playerIdsByAssociationId = new Map(
      playerRows.map((player) => [player.associationId, player.id]),
    );

    const values = records.map((record) => {
      const playerId = playerIdsByAssociationId.get(record.playerAssociationId);

      if (!playerId) {
        throw new Error(`Player ${record.playerAssociationId} does not exist`);
      }

      return {
        absenceReason: record.absenceReason ?? null,
        occurrenceId: record.occurrenceId,
        occurrenceType: record.occurrenceType,
        playerId,
        status: record.status,
      };
    });

    await database
      .insert(participationRecords)
      .values(values)
      .onConflictDoUpdate({
        set: {
          absenceReason: sql`excluded.absence_reason`,
          recordedAt: new Date(),
          status: sql`excluded.status`,
        },
        target: [
          participationRecords.occurrenceId,
          participationRecords.occurrenceType,
          participationRecords.playerId,
        ],
      });

    const storedRecords = await Promise.all(
      records.map((record) => this.findByOccurrence(record.occurrenceType, record.occurrenceId)),
    );
    const storedByPlayer = new Map(
      storedRecords.flat().map((record) => [record.playerAssociationId, record]),
    );

    return records.map((record) => {
      const storedRecord = storedByPlayer.get(record.playerAssociationId);

      if (!storedRecord) {
        throw new Error(`Participation for ${record.playerAssociationId} was not stored`);
      }

      return storedRecord;
    });
  },
});
