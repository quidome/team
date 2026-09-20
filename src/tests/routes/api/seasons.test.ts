import { beforeEach, describe, expect, it, vi } from 'vitest';

const mocks = vi.hoisted(() => ({
  audit: {
    record: vi.fn(),
  },
  seasons: {
    findByStartingYear: vi.fn(),
    save: vi.fn(),
  },
}));

vi.mock('$lib/server/composition-root', () => ({
  currentAuditRepository: () => mocks.audit,
  currentSeasonRepository: () => mocks.seasons,
}));

import { POST } from '../../../routes/api/seasons/+server';

describe('POST /api/seasons', () => {
  beforeEach(() => {
    vi.resetAllMocks();
    mocks.seasons.findByStartingYear.mockResolvedValue(undefined);
    mocks.seasons.save.mockImplementation(async (season) => season);
  });

  it('configures a season from its starting year', async () => {
    const response = await POST({
      request: new Request('http://localhost/api/seasons', {
        body: JSON.stringify({ startingYear: 2026 }),
        headers: { 'content-type': 'application/json' },
        method: 'POST',
      }),
    } as never);

    expect(response.status).toBe(200);
    expect(mocks.audit.record).toHaveBeenCalledWith(
      expect.objectContaining({ action: 'season_configured', entityId: '2026' }),
    );
    await expect(response.json()).resolves.toEqual({ endingYear: 2027, startingYear: 2026 });
    expect(mocks.seasons.save).toHaveBeenCalledWith({ endingYear: 2027, startingYear: 2026 });
  });

  it('rejects a non-integer starting year', async () => {
    const response = await POST({
      request: new Request('http://localhost/api/seasons', {
        body: JSON.stringify({ startingYear: '2026' }),
        headers: { 'content-type': 'application/json' },
        method: 'POST',
      }),
    } as never);

    expect(response.status).toBe(400);
    await expect(response.json()).resolves.toEqual({ error: 'invalid_starting_year' });
    expect(mocks.seasons.save).not.toHaveBeenCalled();
  });
});
