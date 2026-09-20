import { and, asc, eq } from 'drizzle-orm';

import type {
  DutyFairness,
  DutyRepository,
  DutyRequirements,
  DutySlot,
  DutySlotStatus,
  DutyType,
  DutyView,
} from '../../application/duties/duty-repository';
import { createDatabase } from './database';
import { dutyAssignmentHistory, dutyRequirements, dutySignups, dutySlots, players } from './schema';

type Database = ReturnType<typeof createDatabase>;

const dutyTypes: DutyType[] = ['referee', 'jury', 'driving'];

const emptyRequirements: DutyRequirements = {
  drivingSlots: 0,
  jurySlots: 0,
  refereeSlots: 0,
};

const toDutySlot = (row: {
  assignedPlayerAssociationId: string | null;
  dutyType: DutyType;
  id: string;
  occurrenceId: string;
  slotNumber: number;
  status: DutySlotStatus;
}): DutySlot => ({
  ...(row.assignedPlayerAssociationId
    ? { assignedPlayerAssociationId: row.assignedPlayerAssociationId }
    : {}),
  dutyType: row.dutyType,
  id: row.id,
  occurrenceId: row.occurrenceId,
  slotNumber: row.slotNumber,
  status: row.status,
});

const readView = async (database: Database, occurrenceId: string): Promise<DutyView> => {
  const [requirements] = await database
    .select({
      drivingSlots: dutyRequirements.drivingSlots,
      jurySlots: dutyRequirements.jurySlots,
      refereeSlots: dutyRequirements.refereeSlots,
    })
    .from(dutyRequirements)
    .where(eq(dutyRequirements.gameOccurrenceId, occurrenceId))
    .limit(1);
  const slotRows = await database
    .select({
      assignedPlayerAssociationId: players.associationId,
      dutyType: dutySlots.type,
      id: dutySlots.id,
      occurrenceId: dutySlots.gameOccurrenceId,
      slotNumber: dutySlots.slotNumber,
      status: dutySlots.status,
    })
    .from(dutySlots)
    .leftJoin(players, eq(dutySlots.assignedPlayerId, players.id))
    .where(eq(dutySlots.gameOccurrenceId, occurrenceId))
    .orderBy(asc(dutySlots.type), asc(dutySlots.slotNumber));
  const signupRows = await database
    .select({
      dutyType: dutySignups.type,
      occurrenceId: dutySignups.gameOccurrenceId,
      playerAssociationId: players.associationId,
      status: dutySignups.status,
    })
    .from(dutySignups)
    .innerJoin(players, eq(dutySignups.playerId, players.id))
    .where(eq(dutySignups.gameOccurrenceId, occurrenceId))
    .orderBy(asc(players.name));
  const historyRows = await database
    .select({
      dutyType: dutySlots.type,
      occurrenceId: dutySlots.gameOccurrenceId,
      playerAssociationId: players.associationId,
      slotId: dutyAssignmentHistory.slotId,
      status: dutyAssignmentHistory.status,
    })
    .from(dutyAssignmentHistory)
    .innerJoin(dutySlots, eq(dutyAssignmentHistory.slotId, dutySlots.id))
    .innerJoin(players, eq(dutyAssignmentHistory.playerId, players.id))
    .where(eq(dutySlots.gameOccurrenceId, occurrenceId))
    .orderBy(asc(dutyAssignmentHistory.createdAt));

  return {
    assignmentHistory: historyRows,
    fairness: await findFairness(database),
    occurrenceId,
    requirements: requirements ?? emptyRequirements,
    signups: signupRows,
    slots: slotRows.map(toDutySlot),
  };
};

const findFairness = async (database: Database): Promise<DutyFairness[]> => {
  const rows = await database
    .select({
      playerAssociationId: players.associationId,
      status: dutyAssignmentHistory.status,
    })
    .from(dutyAssignmentHistory)
    .innerJoin(players, eq(dutyAssignmentHistory.playerId, players.id));
  const counts = new Map<string, number>();

  for (const row of rows.filter((row) => row.status === 'completed')) {
    counts.set(row.playerAssociationId, (counts.get(row.playerAssociationId) ?? 0) + 1);
  }

  return [...counts.entries()]
    .map(([playerAssociationId, completedCount]) => ({
      completedCount,
      playerAssociationId,
    }))
    .sort((left, right) => left.completedCount - right.completedCount);
};

export const createPostgresDutyRepository = (database: Database): DutyRepository => ({
  async configure(occurrenceId, requirements) {
    await database
      .insert(dutyRequirements)
      .values({
        drivingSlots: requirements.drivingSlots,
        gameOccurrenceId: occurrenceId,
        jurySlots: requirements.jurySlots,
        refereeSlots: requirements.refereeSlots,
      })
      .onConflictDoUpdate({
        set: {
          drivingSlots: requirements.drivingSlots,
          jurySlots: requirements.jurySlots,
          refereeSlots: requirements.refereeSlots,
        },
        target: dutyRequirements.gameOccurrenceId,
      });

    const existingRows = await database
      .select({
        assignedPlayerId: dutySlots.assignedPlayerId,
        id: dutySlots.id,
        slotNumber: dutySlots.slotNumber,
        status: dutySlots.status,
        type: dutySlots.type,
      })
      .from(dutySlots)
      .where(eq(dutySlots.gameOccurrenceId, occurrenceId));
    const counts: Record<DutyType, number> = {
      driving: requirements.drivingSlots,
      jury: requirements.jurySlots,
      referee: requirements.refereeSlots,
    };

    for (const type of dutyTypes) {
      const existingForType = existingRows.filter((row) => row.type === type);

      for (let slotNumber = 1; slotNumber <= counts[type]; slotNumber += 1) {
        const existing = existingForType.find((row) => row.slotNumber === slotNumber);

        if (existing) {
          if (existing.status === 'cancelled') {
            await database
              .update(dutySlots)
              .set({ status: existing.assignedPlayerId ? 'assigned' : 'open' })
              .where(eq(dutySlots.id, existing.id));
          }
          continue;
        }

        await database.insert(dutySlots).values({
          gameOccurrenceId: occurrenceId,
          slotNumber,
          status: 'open',
          type,
        });
      }

      for (const existing of existingForType.filter(
        (row) => row.slotNumber > counts[type] && row.status !== 'cancelled',
      )) {
        await database
          .update(dutySlots)
          .set({ status: 'cancelled' })
          .where(eq(dutySlots.id, existing.id));

        if (existing.assignedPlayerId) {
          await database.insert(dutyAssignmentHistory).values({
            playerId: existing.assignedPlayerId,
            slotId: existing.id,
            status: 'cancelled',
          });
        }
      }
    }

    return readView(database, occurrenceId);
  },

  async findAllSlots() {
    const rows = await database
      .select({
        assignedPlayerAssociationId: players.associationId,
        dutyType: dutySlots.type,
        id: dutySlots.id,
        occurrenceId: dutySlots.gameOccurrenceId,
        slotNumber: dutySlots.slotNumber,
        status: dutySlots.status,
      })
      .from(dutySlots)
      .leftJoin(players, eq(dutySlots.assignedPlayerId, players.id));

    return rows.map(toDutySlot);
  },

  async findByOccurrence(occurrenceId) {
    return readView(database, occurrenceId);
  },

  async findFairness() {
    return findFairness(database);
  },

  async recordSignup(signup) {
    const [player] = await database
      .select({ id: players.id })
      .from(players)
      .where(eq(players.associationId, signup.playerAssociationId))
      .limit(1);

    if (!player) {
      throw new Error(`Player ${signup.playerAssociationId} does not exist`);
    }

    await database
      .insert(dutySignups)
      .values({
        gameOccurrenceId: signup.occurrenceId,
        playerId: player.id,
        status: signup.status,
        type: signup.dutyType,
      })
      .onConflictDoUpdate({
        set: { status: signup.status },
        target: [dutySignups.gameOccurrenceId, dutySignups.type, dutySignups.playerId],
      });

    return readView(database, signup.occurrenceId);
  },

  async assign(slotId, playerAssociationId) {
    const [slot] = await database
      .select({
        assignedPlayerId: dutySlots.assignedPlayerId,
        gameOccurrenceId: dutySlots.gameOccurrenceId,
        status: dutySlots.status,
        type: dutySlots.type,
      })
      .from(dutySlots)
      .where(eq(dutySlots.id, slotId))
      .limit(1);

    if (!slot) {
      throw new Error(`Duty slot ${slotId} does not exist`);
    }

    if (slot.status === 'cancelled' || slot.status === 'completed') {
      throw new Error(`Duty slot ${slotId} cannot be assigned`);
    }

    const [player] = await database
      .select({ id: players.id })
      .from(players)
      .where(eq(players.associationId, playerAssociationId))
      .limit(1);

    if (!player) {
      throw new Error(`Player ${playerAssociationId} does not exist`);
    }

    if (slot.assignedPlayerId === player.id) {
      return readView(database, slot.gameOccurrenceId);
    }

    if (slot.assignedPlayerId) {
      await database.insert(dutyAssignmentHistory).values({
        playerId: slot.assignedPlayerId,
        slotId,
        status: 'reassigned',
      });
      await database
        .update(dutySignups)
        .set({ status: 'volunteer' })
        .where(
          and(
            eq(dutySignups.gameOccurrenceId, slot.gameOccurrenceId),
            eq(dutySignups.type, slot.type),
            eq(dutySignups.playerId, slot.assignedPlayerId),
          ),
        );
    }

    await database
      .insert(dutySignups)
      .values({
        gameOccurrenceId: slot.gameOccurrenceId,
        playerId: player.id,
        status: 'selected',
        type: slot.type,
      })
      .onConflictDoUpdate({
        set: { status: 'selected' },
        target: [dutySignups.gameOccurrenceId, dutySignups.type, dutySignups.playerId],
      });
    await database
      .update(dutySlots)
      .set({ assignedPlayerId: player.id, status: 'assigned' })
      .where(eq(dutySlots.id, slotId));
    await database.insert(dutyAssignmentHistory).values({
      playerId: player.id,
      slotId,
      status: 'assigned',
    });

    return readView(database, slot.gameOccurrenceId);
  },

  async updateSlotStatus(slotId, status) {
    const [slot] = await database
      .select({
        assignedPlayerId: dutySlots.assignedPlayerId,
        gameOccurrenceId: dutySlots.gameOccurrenceId,
        status: dutySlots.status,
      })
      .from(dutySlots)
      .where(eq(dutySlots.id, slotId))
      .limit(1);

    if (!slot) {
      throw new Error(`Duty slot ${slotId} does not exist`);
    }

    if (slot.status === status) {
      return readView(database, slot.gameOccurrenceId);
    }

    await database.update(dutySlots).set({ status }).where(eq(dutySlots.id, slotId));

    if (slot.assignedPlayerId && status !== 'assigned' && status !== 'open') {
      await database.insert(dutyAssignmentHistory).values({
        playerId: slot.assignedPlayerId,
        slotId,
        status,
      });
    }

    return readView(database, slot.gameOccurrenceId);
  },
});
