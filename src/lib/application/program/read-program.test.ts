import { describe, expect, it } from 'vitest';

import { InMemoryGameRepository } from '../../adapters/in-memory-game-repository';
import { InMemoryTrainingSeriesRepository } from '../../adapters/in-memory-training-series-repository';
import { configureGameFixture, scheduleGameOccurrence } from '../games/configure-game';
import { configureTrainingSeries } from '../training/configure-training-series';
import { isAttendanceEligible, readProgram, type ProgramGameEvent } from './read-program';

const gameEvent = (overrides: Partial<ProgramGameEvent> = {}): ProgramGameEvent => ({
  arrivalBufferMinutes: 30,
  awayTeamName: 'U18-1',
  date: '2026-08-25',
  fixtureId: 'fixture-1',
  homeTeamName: 'U16-1',
  id: 'occurrence-1',
  locationName: 'Home court',
  startTime: '19:00',
  status: 'scheduled',
  suggestedDepartureTime: '18:30',
  travelMinutes: 20,
  type: 'game',
  ...overrides,
});

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

describe('isAttendanceEligible', () => {
  it('is true for a scheduled game the team plays at home', () => {
    expect(isAttendanceEligible(gameEvent({ homeTeamName: 'U16-1' }), 'U16-1')).toBe(true);
  });

  it('is true for a scheduled game the team plays away', () => {
    expect(isAttendanceEligible(gameEvent({ awayTeamName: 'U16-1' }), 'U16-1')).toBe(true);
  });

  it('is false for a cancelled game even if the team plays', () => {
    expect(
      isAttendanceEligible(gameEvent({ homeTeamName: 'U16-1', status: 'cancelled' }), 'U16-1'),
    ).toBe(false);
  });

  it('is false for a duty-only game where neither team matches', () => {
    expect(
      isAttendanceEligible(
        gameEvent({ awayTeamName: 'Archipel M16-1', homeTeamName: 'Woodpeckers M16-2' }),
        'U16-1',
      ),
    ).toBe(false);
  });

  it('is false for a training event, since duty-eligibility only applies to games', () => {
    expect(
      isAttendanceEligible(
        {
          date: '2026-08-25',
          durationMinutes: 90,
          id: 'training-1',
          locationName: 'Home court',
          startTime: '18:30',
          status: 'scheduled',
          type: 'training',
        },
        'U16-1',
      ),
    ).toBe(false);
  });
});
