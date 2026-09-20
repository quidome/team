import { beforeEach, describe, expect, it, vi } from 'vitest';

const mocks = vi.hoisted(() => ({
  audit: {
    record: vi.fn(),
  },
  locations: {
    findByName: vi.fn(),
    save: vi.fn(),
  },
}));

vi.mock('$lib/server/composition-root', () => ({
  currentAuditRepository: () => mocks.audit,
  currentLocationRepository: () => mocks.locations,
}));

import { POST } from '../../../routes/api/locations/+server';

describe('POST /api/locations', () => {
  beforeEach(() => {
    vi.resetAllMocks();
    mocks.locations.findByName.mockResolvedValue(undefined);
    mocks.locations.save.mockImplementation(async (location) => location);
  });

  it('configures a reusable location', async () => {
    const response = await POST({
      request: new Request('http://localhost/api/locations', {
        body: JSON.stringify({ name: 'Home court', travelMinutes: 20 }),
        headers: { 'content-type': 'application/json' },
        method: 'POST',
      }),
    } as never);

    expect(response.status).toBe(200);
    expect(mocks.audit.record).toHaveBeenCalledWith(
      expect.objectContaining({ action: 'location_configured', entityId: 'Home court' }),
    );
    await expect(response.json()).resolves.toEqual({ name: 'Home court', travelMinutes: 20 });
  });

  it('rejects a negative travel time', async () => {
    const response = await POST({
      request: new Request('http://localhost/api/locations', {
        body: JSON.stringify({ name: 'Home court', travelMinutes: -1 }),
        headers: { 'content-type': 'application/json' },
        method: 'POST',
      }),
    } as never);

    expect(response.status).toBe(400);
    await expect(response.json()).resolves.toEqual({ error: 'invalid_location' });
    expect(mocks.locations.save).not.toHaveBeenCalled();
  });
});
