import { beforeEach, describe, expect, it, vi } from 'vitest';

import { InMemoryClubRepository } from '$lib/adapters/in-memory-club-repository';

const mocks = vi.hoisted(() => ({
  audit: { record: vi.fn() },
  repositories: {} as { clubs: InMemoryClubRepository },
}));

vi.mock('$lib/server/composition-root', () => ({
  currentAuditRepository: () => mocks.audit,
  currentClubRepository: () => mocks.repositories.clubs,
}));

import { GET, PUT } from '../../../routes/api/club/+server';

const putRequest = (body: unknown) =>
  new Request('http://localhost/api/club', {
    body: JSON.stringify(body),
    headers: { 'content-type': 'application/json' },
    method: 'PUT',
  });

describe('/api/club', () => {
  beforeEach(() => {
    vi.resetAllMocks();
    mocks.repositories.clubs = new InMemoryClubRepository();
  });

  it('returns null when no own club is configured', async () => {
    const response = await GET();

    await expect(response.json()).resolves.toBeNull();
  });

  it('configures the own club with its source club ID', async () => {
    const response = await PUT({
      request: putRequest({ name: 'Blue Drakes', sourceClubId: 37 }),
    } as never);

    expect(response.status).toBe(200);
    await expect(response.json()).resolves.toEqual({
      isOwnClub: true,
      name: 'Blue Drakes',
      sourceClubId: 37,
    });
    expect(mocks.audit.record).toHaveBeenCalledWith(
      expect.objectContaining({ action: 'own_club_configured', entityId: 'Blue Drakes' }),
    );

    const current = await GET();
    await expect(current.json()).resolves.toEqual({
      isOwnClub: true,
      name: 'Blue Drakes',
      sourceClubId: 37,
      teamNames: [],
    });
  });

  it('rejects an invalid source club ID', async () => {
    const response = await PUT({
      request: putRequest({ name: 'Blue Drakes', sourceClubId: 'thirty-seven' }),
    } as never);

    expect(response.status).toBe(400);
    await expect(response.json()).resolves.toEqual({ error: 'invalid_club' });
  });

  it('rejects a second own club', async () => {
    await PUT({ request: putRequest({ name: 'Blue Drakes' }) } as never);

    const response = await PUT({ request: putRequest({ name: 'Woodpeckers' }) } as never);

    expect(response.status).toBe(400);
    await expect(response.json()).resolves.toEqual({
      error: 'Own club is already configured as Blue Drakes',
    });
  });
});
