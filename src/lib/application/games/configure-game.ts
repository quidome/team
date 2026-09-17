import type {
  GameFixture,
  GameOccurrence,
  GameRepository,
  StoredGameFixture,
  StoredGameOccurrence,
} from './game-repository';

export const configureGameFixture = async (
  games: GameRepository,
  fixture: GameFixture,
): Promise<StoredGameFixture> => games.saveFixture(fixture);

export const scheduleGameOccurrence = async (
  games: GameRepository,
  fixtureId: string,
  occurrence: GameOccurrence,
): Promise<StoredGameOccurrence> => games.saveOccurrence(fixtureId, occurrence);

export const cancelGameOccurrence = async (
  games: GameRepository,
  occurrenceId: string,
): Promise<StoredGameOccurrence> => {
  const occurrence = await games.findOccurrenceById(occurrenceId);

  if (!occurrence) {
    throw new Error(`Game occurrence ${occurrenceId} does not exist`);
  }

  return games.updateOccurrenceStatus(occurrenceId, 'cancelled');
};

export const rescheduleGameOccurrence = async (
  games: GameRepository,
  occurrenceId: string,
  occurrence: GameOccurrence,
): Promise<StoredGameOccurrence> => {
  const existingOccurrence = await games.findOccurrenceById(occurrenceId);

  if (!existingOccurrence) {
    throw new Error(`Game occurrence ${occurrenceId} does not exist`);
  }

  const replacement = await games.saveOccurrence(existingOccurrence.fixtureId, occurrence);

  await games.updateOccurrenceStatus(occurrenceId, 'cancelled');

  return replacement;
};
