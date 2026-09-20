import { describe, expect, it } from 'vitest';

import { InMemoryGameRepository } from '../../adapters/in-memory-game-repository';
import { InMemoryTrainingSeriesRepository } from '../../adapters/in-memory-training-series-repository';
import { configureGameFixture, scheduleGameOccurrence } from '../games/configure-game';
import { configureTrainingSeries } from '../training/configure-training-series';
import { readProgram } from './read-program';

describe('read program', () => {
  it('combines games and generated training occurrences chronologically', async () => {
    const games = new InMemoryGameRepository();
    const trainingSeries = new InMemoryTrainingSeriesRepository();
    const fixture = await configureGameFixture(games, {
      awayTeamName: 'U18-1',
      homeTeamName: 'U16-1',
    });

    await scheduleGameOccurrence(games, fixture.id, {
      arrivalBufferMinutes: 30,
      date: '2026-08-25',
      locationName: 'Away court',
      startTime: '19:00',
      travelMinutes: 20,
    });
    await configureTrainingSeries(trainingSeries, {
      durationMinutes: 90,
      endDate: '2026-09-01',
      locationName: 'Home court',
      startDate: '2026-08-18',
      startTime: '18:30',
      weekday: 2,
    });
    await trainingSeries.saveOccurrence({
      date: '2026-08-20',
      durationMinutes: 60,
      locationName: 'Away court',
      startTime: '17:00',
    });

    await expect(readProgram(games, trainingSeries)).resolves.toEqual([
      expect.objectContaining({ date: '2026-08-18', type: 'training' }),
      expect.objectContaining({ date: '2026-08-20', startTime: '17:00', type: 'training' }),
      expect.objectContaining({ date: '2026-08-25', startTime: '18:30', type: 'training' }),
      expect.objectContaining({ date: '2026-08-25', startTime: '19:00', type: 'game' }),
      expect.objectContaining({ date: '2026-09-01', type: 'training' }),
    ]);
  });
});
