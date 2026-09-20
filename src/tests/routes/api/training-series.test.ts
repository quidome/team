import { beforeEach, describe, expect, it, vi } from 'vitest';

const mocks = vi.hoisted(() => ({
  audit: {
    record: vi.fn(),
  },
  trainingSeries: {
    findById: vi.fn(),
    save: vi.fn(),
  },
}));

vi.mock('$lib/server/composition-root', () => ({
  currentAuditRepository: () => mocks.audit,
  currentTrainingSeriesRepository: () => mocks.trainingSeries,
}));

import { POST } from '../../../routes/api/training-series/+server';

describe('POST /api/training-series', () => {
  beforeEach(() => {
    vi.resetAllMocks();
    mocks.trainingSeries.save.mockImplementation(async (series, occurrences) => ({
      id: 'training-series-1',
      occurrences,
      series,
    }));
  });

  it('stores a recurring training series and its generated occurrences', async () => {
    const response = await POST({
      request: new Request('http://localhost/api/training-series', {
        body: JSON.stringify({
          durationMinutes: 90,
          endDate: '2026-09-01',
          locationName: 'Home court',
          startDate: '2026-08-18',
          startTime: '18:30',
          weekday: 2,
        }),
        headers: { 'content-type': 'application/json' },
        method: 'POST',
      }),
    } as never);

    expect(response.status).toBe(200);
    expect(mocks.audit.record).toHaveBeenCalledWith(
      expect.objectContaining({
        action: 'training_series_configured',
        entityId: 'training-series-1',
      }),
    );
    await expect(response.json()).resolves.toMatchObject({
      id: 'training-series-1',
      occurrences: expect.arrayContaining([
        expect.objectContaining({ date: '2026-08-18' }),
        expect.objectContaining({ date: '2026-08-25' }),
        expect.objectContaining({ date: '2026-09-01' }),
      ]),
    });
  });

  it('rejects an invalid start time', async () => {
    const response = await POST({
      request: new Request('http://localhost/api/training-series', {
        body: JSON.stringify({
          durationMinutes: 90,
          endDate: '2026-09-01',
          locationName: 'Home court',
          startDate: '2026-08-18',
          startTime: '25:00',
          weekday: 2,
        }),
        headers: { 'content-type': 'application/json' },
        method: 'POST',
      }),
    } as never);

    expect(response.status).toBe(400);
    await expect(response.json()).resolves.toEqual({ error: 'invalid_training_series' });
    expect(mocks.trainingSeries.save).not.toHaveBeenCalled();
  });
});
