import { suggestDepartureTime } from '../../domain/game';

export { suggestDepartureTime };

export type GameOccurrenceStatus = 'cancelled' | 'scheduled';
export type SeasonHalf = 'H1' | 'H2';

export interface GameFixture {
  awayTeamName: string;
  homeTeamName: string;
  isHome?: boolean;
  opponentAddress?: string;
  opponentName?: string;
  opponentTravelMinutes?: number;
  ourTeamName?: string;
  seasonHalf?: SeasonHalf;
  seasonStartingYear?: number;
}

export interface GameOccurrence {
  arrivalBufferMinutes: number;
  date: string;
  locationName: string;
  startTime: string;
  travelMinutes: number;
}

export interface StoredGameFixture {
  fixture: GameFixture;
  id: string;
}

export interface StoredGameProgramOccurrence extends StoredGameOccurrence {
  awayTeamName: string;
  homeTeamName: string;
  seasonHalf?: SeasonHalf;
}

export interface StoredGameOccurrence extends GameOccurrence {
  fixtureId: string;
  id: string;
  status: GameOccurrenceStatus;
  suggestedDepartureTime: string;
}

export interface GameRepository {
  findAllOccurrences(): Promise<StoredGameProgramOccurrence[]>;
  findFixtureById(id: string): Promise<StoredGameFixture | undefined>;
  findOccurrenceById(id: string): Promise<StoredGameOccurrence | undefined>;
  findOccurrences(fixtureId: string): Promise<StoredGameOccurrence[]>;
  saveFixture(fixture: GameFixture): Promise<StoredGameFixture>;
  saveOccurrence(fixtureId: string, occurrence: GameOccurrence): Promise<StoredGameOccurrence>;
  updateOccurrence(id: string, occurrence: GameOccurrence): Promise<StoredGameOccurrence>;
  updateOccurrenceStatus(id: string, status: GameOccurrenceStatus): Promise<StoredGameOccurrence>;
}
