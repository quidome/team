import { describe, expect, it } from 'vitest';

import { calculateNormalAgeGroup } from '../../domain/normal-age-group';
import { readHistory } from './read-history';

describe('readHistory', () => {
  it('summarizes participation percentages and completed duties for active team players', () => {
    const report = readHistory(
      [
        {
          associationId: 'p-1',
          birthDate: '2010-01-01',
          membership: {
            participationType: 'trains_and_plays',
            playerAssociationId: 'p-1',
            relationship: 'primary',
            seasonStartingYear: 2026,
            status: 'active',
            teamName: 'U16-1',
          },
          name: 'Alex',
          normalAgeGroup: calculateNormalAgeGroup(2026, 2010),
        },
        {
          associationId: 'p-2',
          birthDate: '2010-01-01',
          membership: {
            participationType: 'trains_only',
            playerAssociationId: 'p-2',
            relationship: 'primary',
            seasonStartingYear: 2026,
            status: 'active',
            teamName: 'U16-1',
          },
          name: 'Robin',
          normalAgeGroup: calculateNormalAgeGroup(2026, 2010),
        },
      ],
      [
        {
          occurrenceId: 'g-1',
          occurrenceType: 'game',
          playerAssociationId: 'p-1',
          status: 'present',
        },
        {
          occurrenceId: 'g-2',
          occurrenceType: 'game',
          playerAssociationId: 'p-1',
          status: 'absent',
        },
        {
          occurrenceId: 't-1',
          occurrenceType: 'training',
          playerAssociationId: 'p-2',
          status: 'present',
        },
      ],
      [{ completedCount: 2, playerAssociationId: 'p-1' }],
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
        },
      ],
    );

    expect(report.players).toEqual([
      expect.objectContaining({
        dutiesCompleted: 2,
        games: { percentage: 50, present: 1, recorded: 2 },
        name: 'Alex',
      }),
      expect.objectContaining({
        games: { present: 0, recorded: 0 },
        name: 'Robin',
        trainings: { percentage: 100, present: 1, recorded: 1 },
      }),
    ]);
    expect(report.entries[0]).toMatchObject({
      eventLabel: 'U16-1 vs U18-1',
      playerAssociationId: 'p-1',
    });
  });
});
