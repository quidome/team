import { describe, expect, it } from 'vitest';

import { calculateNormalAgeGroup } from '../../domain/normal-age-group';
import { readHistory } from './read-history';

describe('readHistory', () => {
  it('summarizes participation percentages and completed duties for active team players', () => {
    const report = readHistory(
      [
        {
          birthDate: '2010-01-01',
          firstName: 'Alex',
          id: 'p-1',
          membership: {
            participationType: 'trains_and_plays',
            playerId: 'p-1',
            relationship: 'primary',
            seasonStartingYear: 2026,
            status: 'active',
            teamName: 'U16-1',
          },
          normalAgeGroup: calculateNormalAgeGroup(2026, 2010),
        },
        {
          birthDate: '2010-01-01',
          firstName: 'Robin',
          id: 'p-2',
          membership: {
            participationType: 'trains_only',
            playerId: 'p-2',
            relationship: 'primary',
            seasonStartingYear: 2026,
            status: 'active',
            teamName: 'U16-1',
          },
          normalAgeGroup: calculateNormalAgeGroup(2026, 2010),
        },
      ],
      [
        {
          occurrenceId: 'g-1',
          occurrenceType: 'game',
          playerId: 'p-1',
          status: 'present',
        },
        {
          occurrenceId: 'g-2',
          occurrenceType: 'game',
          playerId: 'p-1',
          status: 'absent',
        },
        {
          occurrenceId: 't-1',
          occurrenceType: 'training',
          playerId: 'p-2',
          status: 'present',
        },
      ],
      [{ completedCount: 2, playerId: 'p-1' }],
      [
        {
          awayTeamName: 'U18-1',
          date: '2026-08-15',
          homeTeamName: 'U16-1',
          id: 'g-1',
          locationName: 'Away court',
          startTime: '14:30',
          status: 'scheduled',
          suggestedDepartureTime: '13:40',
          travelMinutes: 20,
          arrivalBufferMinutes: 30,
          fixtureId: 'fixture-1',
          type: 'game',
        },
        {
          date: '2026-08-12',
          durationMinutes: 90,
          id: 't-1',
          locationName: 'Home court',
          seriesId: 'series-1',
          startTime: '18:00',
          type: 'training',
          occurrenceId: 't-1',
          status: 'scheduled',
        },
      ],
    );

    expect(report.players).toEqual([
      expect.objectContaining({
        dutiesCompleted: 2,
        firstName: 'Alex',
        games: { denominator: 2, percentage: 50, present: 1, recorded: 2 },
      }),
      expect.objectContaining({
        firstName: 'Robin',
        games: { denominator: 0, present: 0, recorded: 0 },
        trainings: { denominator: 1, percentage: 100, present: 1, recorded: 1 },
      }),
    ]);
    expect(report.entries[0]).toMatchObject({
      eventLabel: 'U16-1 vs U18-1',
      playerId: 'p-1',
    });
  });

  it('can use scheduled eligible occurrences as the denominator', () => {
    const report = readHistory(
      [
        {
          birthDate: '2010-01-01',
          firstName: 'Alex',
          id: 'p-1',
          membership: {
            participationType: 'trains_and_plays',
            playerId: 'p-1',
            relationship: 'primary',
            seasonStartingYear: 2026,
            status: 'active',
            teamName: 'U16-1',
          },
          normalAgeGroup: calculateNormalAgeGroup(2026, 2010),
        },
      ],
      [
        {
          occurrenceId: 't-1',
          occurrenceType: 'training',
          playerId: 'p-1',
          status: 'present',
        },
      ],
      [],
      [
        {
          date: '2026-08-12',
          durationMinutes: 90,
          id: 't-1',
          locationName: 'Home court',
          startTime: '18:00',
          status: 'scheduled',
          type: 'training',
          occurrenceId: 't-1',
        },
        {
          date: '2026-08-19',
          durationMinutes: 90,
          id: 't-2',
          locationName: 'Home court',
          startTime: '18:00',
          status: 'scheduled',
          type: 'training',
          occurrenceId: 't-2',
        },
        {
          date: '2026-08-26',
          durationMinutes: 90,
          id: 't-3',
          locationName: 'Home court',
          startTime: '18:00',
          status: 'cancelled',
          type: 'training',
          occurrenceId: 't-3',
        },
      ],
      'U16-1',
      'scheduled',
    );

    expect(report.denominator).toBe('scheduled');
    expect(report.players[0]?.trainings).toEqual({
      denominator: 2,
      percentage: 50,
      present: 1,
      recorded: 1,
    });
  });
});
