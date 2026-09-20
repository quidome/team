import { beforeEach, describe, expect, it, vi } from 'vitest';

const mocks = vi.hoisted(() => ({
  checkDatabaseConnection: vi.fn(),
}));

vi.mock('$lib/server/composition-root', () => ({
  checkDatabaseConnection: mocks.checkDatabaseConnection,
}));

import { GET as liveness } from '../../../routes/api/health/liveness/+server';
import { GET as readiness } from '../../../routes/api/health/readiness/+server';

describe('health endpoints', () => {
  beforeEach(() => {
    vi.resetAllMocks();
  });

  it('reports liveness without checking dependencies', async () => {
    const response = liveness();

    expect(response.status).toBe(200);
    await expect(response.json()).resolves.toEqual({ status: 'ok' });
    expect(mocks.checkDatabaseConnection).not.toHaveBeenCalled();
  });

  it('reports readiness when the database is reachable', async () => {
    mocks.checkDatabaseConnection.mockResolvedValue(true);

    const response = await readiness();

    expect(response.status).toBe(200);
    await expect(response.json()).resolves.toEqual({ status: 'ready' });
  });

  it('returns service unavailable when the database is unreachable', async () => {
    mocks.checkDatabaseConnection.mockResolvedValue(false);

    const response = await readiness();

    expect(response.status).toBe(503);
    await expect(response.json()).resolves.toEqual({ status: 'not_ready' });
  });
});
