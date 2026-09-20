import { beforeEach, describe, expect, it, vi } from 'vitest';

const mocks = vi.hoisted(() => ({
  audit: {
    record: vi.fn(),
  },
  players: {
    findByAssociationId: vi.fn(),
    save: vi.fn(),
  },
}));

vi.mock('$lib/server/composition-root', () => ({
  currentAuditRepository: () => mocks.audit,
  currentPlayerRepository: () => mocks.players,
}));

import { POST } from '../../../routes/api/players/+server';

describe('POST /api/players', () => {
  beforeEach(() => {
    vi.resetAllMocks();
    mocks.players.findByAssociationId.mockResolvedValue(undefined);
    mocks.players.save.mockImplementation(async (player) => player);
  });

  it('registers a player with name, birthday, and association ID', async () => {
    const response = await POST({
      request: new Request('http://localhost/api/players', {
        body: JSON.stringify({
          associationId: '12345',
          birthDate: '2011-06-15',
          name: 'Avery',
        }),
        headers: { 'content-type': 'application/json' },
        method: 'POST',
      }),
    } as never);

    expect(response.status).toBe(200);
    expect(mocks.audit.record).toHaveBeenCalledWith(
      expect.objectContaining({ action: 'player_configured', entityId: '12345' }),
    );
    await expect(response.json()).resolves.toEqual({
      associationId: '12345',
      birthDate: '2011-06-15',
      name: 'Avery',
    });
  });

  it('rejects a malformed birthday', async () => {
    const response = await POST({
      request: new Request('http://localhost/api/players', {
        body: JSON.stringify({
          associationId: '12345',
          birthDate: '15-06-2011',
          name: 'Avery',
        }),
        headers: { 'content-type': 'application/json' },
        method: 'POST',
      }),
    } as never);

    expect(response.status).toBe(400);
    await expect(response.json()).resolves.toEqual({ error: 'invalid_player' });
    expect(mocks.players.save).not.toHaveBeenCalled();
  });
});
