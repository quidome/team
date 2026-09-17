import { describe, expect, it } from 'vitest';

import { generateTrainingOccurrences } from './training-series';

describe('recurring training occurrences', () => {
  it('generates the configured weekday through the inclusive end date', () => {
    const occurrences = generateTrainingOccurrences({
      durationMinutes: 90,
      endDate: '2026-09-01',
      locationName: 'Home court',
      startDate: '2026-08-18',
      startTime: '18:30',
      weekday: 2,
    });

    expect(occurrences).toEqual([
      {
        date: '2026-08-18',
        durationMinutes: 90,
        locationName: 'Home court',
        startTime: '18:30',
      },
      {
        date: '2026-08-25',
        durationMinutes: 90,
        locationName: 'Home court',
        startTime: '18:30',
      },
      {
        date: '2026-09-01',
        durationMinutes: 90,
        locationName: 'Home court',
        startTime: '18:30',
      },
    ]);
  });

  it('returns no occurrences when the series starts after its end date', () => {
    expect(
      generateTrainingOccurrences({
        durationMinutes: 90,
        endDate: '2026-08-17',
        locationName: 'Home court',
        startDate: '2026-08-18',
        startTime: '18:30',
        weekday: 2,
      }),
    ).toEqual([]);
  });
});
