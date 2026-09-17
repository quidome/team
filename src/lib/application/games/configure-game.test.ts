import { describe, expect, it } from 'vitest';

import { InMemoryGameRepository } from '../../adapters/in-memory-game-repository';
import {
  configureGameFixture,
  rescheduleGameOccurrence,
  scheduleGameOccurrence,
} from './configure-game';

describe('game fixtures and occurrences', () => {
  it('stores a stable fixture separately from its scheduled occurrence', async () => {
    const games = new InMemoryGameRepository();
    const fixture = await configureGameFixture(games, {
      awayTeamName: 'U18-1',
      homeTeamName: 'U16-1',
    });

    const occurrence = await scheduleGameOccurrence(games, fixture.id, {
      arrivalBufferMinutes: 30,
      date: '2026-09-05',
      locationName: 'Home court',
      startTime: '14:30',
      travelMinutes: 0,
    });

    expect(fixture.fixture).toEqual({ awayTeamName: 'U18-1', homeTeamName: 'U16-1' });
    expect(occurrence).toMatchObject({
      date: '2026-09-05',
      fixtureId: fixture.id,
      startTime: '14:30',
      suggestedDepartureTime: '14:00',
    });
    await expect(games.findOccurrences(fixture.id)).resolves.toHaveLength(1);
  });

  it('cancels the original occurrence when rescheduling a fixture', async () => {
    const games = new InMemoryGameRepository();
    const fixture = await configureGameFixture(games, {
      awayTeamName: 'U18-1',
      homeTeamName: 'U16-1',
    });
    const original = await scheduleGameOccurrence(games, fixture.id, {
      arrivalBufferMinutes: 30,
      date: '2026-09-05',
      locationName: 'Home court',
      startTime: '14:30',
      travelMinutes: 0,
    });

    const replacement = await rescheduleGameOccurrence(games, original.id, {
      arrivalBufferMinutes: 30,
      date: '2026-09-12',
      locationName: 'Home court',
      startTime: '15:00',
      travelMinutes: 0,
    });

    expect(replacement.date).toBe('2026-09-12');
    await expect(games.findOccurrenceById(original.id)).resolves.toMatchObject({
      status: 'cancelled',
    });
    await expect(games.findOccurrences(fixture.id)).resolves.toEqual(
      expect.arrayContaining([
        expect.objectContaining({ date: '2026-09-05', status: 'cancelled' }),
        expect.objectContaining({ date: '2026-09-12', status: 'scheduled' }),
      ]),
    );
  });
});
