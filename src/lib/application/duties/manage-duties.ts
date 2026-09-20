import type { GameRepository } from '../games/game-repository';
import type {
  DutyRepository,
  DutySlotStatus,
  DutyView,
  DutyRequirements,
  DutySignup,
} from './duty-repository';

export const configureDuties = (
  duties: DutyRepository,
  occurrenceId: string,
  requirements: DutyRequirements,
): Promise<DutyView> => duties.configure(occurrenceId, requirements);

export const recordDutySignup = (duties: DutyRepository, signup: DutySignup): Promise<DutyView> =>
  duties.recordSignup(signup);

export const assignDuty = (
  duties: DutyRepository,
  slotId: string,
  playerAssociationId: string,
): Promise<DutyView> => duties.assign(slotId, playerAssociationId);

export const correctDutyStatus = (
  duties: DutyRepository,
  slotId: string,
  status: DutySlotStatus,
): Promise<DutyView> => duties.updateSlotStatus(slotId, status);

const occurrenceTimestamp = (date: string, startTime: string): number =>
  new Date(`${date}T${startTime}:00.000Z`).getTime();

export const completeDueDuties = async (
  duties: DutyRepository,
  games: GameRepository,
  now = new Date(),
): Promise<void> => {
  const [slots, occurrences] = await Promise.all([
    duties.findAllSlots(),
    games.findAllOccurrences(),
  ]);
  const occurrenceDetails = new Map(
    occurrences.map((occurrence) => [
      occurrence.id,
      {
        status: occurrence.status,
        timestamp: occurrenceTimestamp(occurrence.date, occurrence.startTime),
      },
    ]),
  );

  await Promise.all(
    slots
      .filter((slot) => {
        const occurrence = occurrenceDetails.get(slot.occurrenceId);

        return (
          occurrence !== undefined &&
          (occurrence.status === 'cancelled'
            ? slot.status === 'open' || slot.status === 'assigned'
            : slot.status === 'assigned' && occurrence.timestamp <= now.getTime())
        );
      })
      .map((slot) =>
        duties.updateSlotStatus(
          slot.id,
          occurrenceDetails.get(slot.occurrenceId)?.status === 'cancelled'
            ? 'cancelled'
            : 'completed',
        ),
      ),
  );
};

export const readDuties = async (
  duties: DutyRepository,
  games: GameRepository,
  occurrenceId: string,
): Promise<DutyView> => {
  await completeDueDuties(duties, games);

  return duties.findByOccurrence(occurrenceId);
};
