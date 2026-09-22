import { beforeEach, describe, expect, it, vi } from 'vitest';

const mocks = vi.hoisted(() => ({
  audit: {
    record: vi.fn(),
  },
  players: {
    findByAssociationId: vi.fn(),
    save: vi.fn(),
    update: vi.fn(),
  },
}));

vi.mock('$lib/server/composition-root', () => ({
  currentAuditRepository: () => mocks.audit,
  currentPlayerRepository: () => mocks.players,
}));

import { POST, PUT } from '../../../routes/api/players/+server';

describe('POST /api/players', () => {
  beforeEach(() => {
    vi.resetAllMocks();
    mocks.players.findByAssociationId.mockResolvedValue(undefined);
    mocks.players.save.mockImplementation(async (player) => ({ ...player, id: 'player-1' }));
  });

  it('registers a player with first name, last name, birthday, and association ID', async () => {
    const response = await POST({
      request: new Request('http://localhost/api/players', {
        body: JSON.stringify({
          associationId: '12345',
          birthDate: '2011-06-15',
          firstName: 'Avery',
          lastName: 'Smith',
        }),
        headers: { 'content-type': 'application/json' },
        method: 'POST',
      }),
    } as never);

    expect(response.status).toBe(200);
    expect(mocks.audit.record).toHaveBeenCalledWith(
      expect.objectContaining({ action: 'player_configured', entityId: 'player-1' }),
    );
    await expect(response.json()).resolves.toEqual({
      associationId: '12345',
      birthDate: '2011-06-15',
      firstName: 'Avery',
      id: 'player-1',
      lastName: 'Smith',
    });
  });

  it('registers a player with only a first name', async () => {
    const response = await POST({
      request: new Request('http://localhost/api/players', {
        body: JSON.stringify({ firstName: 'Avery' }),
        headers: { 'content-type': 'application/json' },
        method: 'POST',
      }),
    } as never);

    expect(response.status).toBe(200);
    await expect(response.json()).resolves.toEqual({ firstName: 'Avery', id: 'player-1' });
  });

  it('rejects a malformed birthday', async () => {
    const response = await POST({
      request: new Request('http://localhost/api/players', {
        body: JSON.stringify({
          associationId: '12345',
          birthDate: '15-06-2011',
          firstName: 'Avery',
        }),
        headers: { 'content-type': 'application/json' },
        method: 'POST',
      }),
    } as never);

    expect(response.status).toBe(400);
    await expect(response.json()).resolves.toEqual({ error: 'invalid_player' });
    expect(mocks.players.save).not.toHaveBeenCalled();
  });

  it('rejects a missing first name', async () => {
    const response = await POST({
      request: new Request('http://localhost/api/players', {
        body: JSON.stringify({ associationId: '12345' }),
        headers: { 'content-type': 'application/json' },
        method: 'POST',
      }),
    } as never);

    expect(response.status).toBe(400);
    await expect(response.json()).resolves.toEqual({ error: 'invalid_player' });
    expect(mocks.players.save).not.toHaveBeenCalled();
  });
});

describe('PUT /api/players', () => {
  beforeEach(() => {
    vi.resetAllMocks();
    mocks.players.findByAssociationId.mockResolvedValue(undefined);
    mocks.players.update.mockImplementation(async (player) => player);
  });

  it('updates an existing player', async () => {
    const response = await PUT({
      request: new Request('http://localhost/api/players', {
        body: JSON.stringify({
          associationId: '12345',
          birthDate: '2011-06-15',
          firstName: 'Avery',
          id: 'player-1',
        }),
        headers: { 'content-type': 'application/json' },
        method: 'PUT',
      }),
    } as never);

    expect(response.status).toBe(200);
    expect(mocks.audit.record).toHaveBeenCalledWith(
      expect.objectContaining({ action: 'player_updated', entityId: 'player-1' }),
    );
    await expect(response.json()).resolves.toEqual({
      associationId: '12345',
      birthDate: '2011-06-15',
      firstName: 'Avery',
      id: 'player-1',
    });
  });

  it('rejects a malformed body', async () => {
    const response = await PUT({
      request: new Request('http://localhost/api/players', {
        body: JSON.stringify({ id: 'player-1' }),
        headers: { 'content-type': 'application/json' },
        method: 'PUT',
      }),
    } as never);

    expect(response.status).toBe(400);
    await expect(response.json()).resolves.toEqual({ error: 'invalid_player' });
    expect(mocks.players.update).not.toHaveBeenCalled();
  });

  it('reports a nonexistent player as not found', async () => {
    mocks.players.update.mockRejectedValue(new Error('Player does not exist'));

    const response = await PUT({
      request: new Request('http://localhost/api/players', {
        body: JSON.stringify({ firstName: 'Avery', id: 'missing-player' }),
        headers: { 'content-type': 'application/json' },
        method: 'PUT',
      }),
    } as never);

    expect(response.status).toBe(400);
    await expect(response.json()).resolves.toEqual({ error: 'player_not_found' });
  });
});
