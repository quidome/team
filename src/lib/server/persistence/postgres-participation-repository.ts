import { and, eq, inArray, sql } from 'drizzle-orm';

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
        playerId: participationRecords.playerId,
        status: participationRecords.status,
      })
      .from(participationRecords);

    return records.map((record) => ({
      ...(record.absenceReason === null ? {} : { absenceReason: record.absenceReason }),
      occurrenceId: record.occurrenceId,
      occurrenceType: record.occurrenceType,
      playerId: record.playerId,
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
        playerId: participationRecords.playerId,
        status: participationRecords.status,
      })
      .from(participationRecords)
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
      playerId: record.playerId,
      status: record.status,
    }));
  },

  async saveMany(records: ParticipationRecord[]): Promise<ParticipationRecord[]> {
    if (records.length === 0) {
      return [];
    }

    const existingRows = await database
      .select({ id: players.id })
      .from(players)
      .where(
        inArray(
          players.id,
          records.map((record) => record.playerId),
        ),
      );
    const existingIds = new Set(existingRows.map((row) => row.id));

    const values = records.map((record) => {
      if (!existingIds.has(record.playerId)) {
        throw new Error(`Player ${record.playerId} does not exist`);
      }

      return {
        absenceReason: record.absenceReason ?? null,
        occurrenceId: record.occurrenceId,
        occurrenceType: record.occurrenceType,
        playerId: record.playerId,
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
    const storedByPlayer = new Map(storedRecords.flat().map((record) => [record.playerId, record]));

    return records.map((record) => {
      const storedRecord = storedByPlayer.get(record.playerId);

      if (!storedRecord) {
        throw new Error(`Participation for ${record.playerId} was not stored`);
      }

      return storedRecord;
    });
  },
});
