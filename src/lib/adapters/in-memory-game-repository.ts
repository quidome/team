import { suggestDepartureTime } from '../domain/game';
import type {
  GameFixture,
  GameOccurrence,
  GameRepository,
  StoredGameFixture,
  StoredGameOccurrence,
} from '../application/games/game-repository';

export class InMemoryGameRepository implements GameRepository {
  private nextFixtureId = 1;
  private nextOccurrenceId = 1;
  private readonly fixtures = new Map<string, StoredGameFixture>();
  private readonly occurrences = new Map<string, StoredGameOccurrence[]>();

  async findFixtureById(id: string): Promise<StoredGameFixture | undefined> {
    return this.fixtures.get(id);
  }

  async findOccurrenceById(id: string): Promise<StoredGameOccurrence | undefined> {
    for (const occurrences of this.occurrences.values()) {
      const occurrence = occurrences.find((candidate) => candidate.id === id);

      if (occurrence) {
        return occurrence;
      }
    }

    return undefined;
  }

  async findOccurrences(fixtureId: string): Promise<StoredGameOccurrence[]> {
    return this.occurrences.get(fixtureId) ?? [];
  }

  async saveFixture(fixture: GameFixture): Promise<StoredGameFixture> {
    const storedFixture = { fixture, id: `game-fixture-${this.nextFixtureId++}` };

    this.fixtures.set(storedFixture.id, storedFixture);

    return storedFixture;
  }

  async saveOccurrence(
    fixtureId: string,
    occurrence: GameOccurrence,
  ): Promise<StoredGameOccurrence> {
    if (!this.fixtures.has(fixtureId)) {
      throw new Error(`Game fixture ${fixtureId} does not exist`);
    }

    const storedOccurrence = {
      ...occurrence,
      fixtureId,
      id: `game-occurrence-${this.nextOccurrenceId++}`,
      status: 'scheduled' as const,
      suggestedDepartureTime: suggestDepartureTime(
        occurrence.startTime,
        occurrence.travelMinutes,
        occurrence.arrivalBufferMinutes,
      ),
    };
    const fixtureOccurrences = this.occurrences.get(fixtureId) ?? [];

    fixtureOccurrences.push(storedOccurrence);
    this.occurrences.set(fixtureId, fixtureOccurrences);

    return storedOccurrence;
  }

  async updateOccurrenceStatus(
    id: string,
    status: 'cancelled' | 'scheduled',
  ): Promise<StoredGameOccurrence> {
    const occurrence = await this.findOccurrenceById(id);

    if (!occurrence) {
      throw new Error(`Game occurrence ${id} does not exist`);
    }

    occurrence.status = status;

    return occurrence;
  }
}
