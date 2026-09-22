import { beforeEach, describe, expect, it, vi } from 'vitest';

const mocks = vi.hoisted(() => ({
  audit: {
    record: vi.fn(),
  },
  memberships: {
    find: vi.fn(),
    findAll: vi.fn(),
    save: vi.fn(),
    update: vi.fn(),
  },
  players: {
    deleteById: vi.fn(),
    findAll: vi.fn(),
    findByAssociationId: vi.fn(),
    findById: vi.fn(),
    save: vi.fn(),
    update: vi.fn(),
  },
  settings: {
    get: vi.fn(),
  },
}));

vi.mock('$lib/server/composition-root', () => ({
  currentCoordinatorSettingsRepository: () => mocks.settings,
  currentMembershipRepository: () => mocks.memberships,
  currentPlayerRepository: () => mocks.players,
  withCurrentImportTransaction: async (work: (repositories: unknown) => unknown) =>
    work({
      audit: mocks.audit,
      memberships: mocks.memberships,
      players: mocks.players,
    }),
}));

import { POST } from '../../../routes/api/imports/players/+server';

describe('POST /api/imports/players', () => {
  beforeEach(() => {
    vi.resetAllMocks();
    mocks.settings.get.mockResolvedValue({ primaryTeamName: 'U16-1', seasonStartingYear: 2026 });
    mocks.players.findAll.mockResolvedValue([]);
    mocks.memberships.findAll.mockResolvedValue([]);
    mocks.players.findByAssociationId.mockResolvedValue(undefined);
    mocks.players.save.mockImplementation(async (player) => ({ ...player, id: 'player-1' }));
    mocks.memberships.find.mockResolvedValue(undefined);
    mocks.memberships.save.mockImplementation(async (membership) => membership);
  });

  it('imports a valid mapped CSV', async () => {
    const response = await POST({
      request: new Request('http://localhost/api/imports/players', {
        body: JSON.stringify({
          content: 'first,last\nAvery,Stone',
          encoding: 'text',
          fileName: 'roster.csv',
          mapping: { firstName: 'first', lastName: 'last' },
          sourceName: 'roster.csv',
        }),
        headers: { 'content-type': 'application/json' },
        method: 'POST',
      }),
    } as never);

    expect(response.status).toBe(200);
    await expect(response.json()).resolves.toEqual(
      expect.objectContaining({
        failed: [],
        imported: [expect.objectContaining({ playerId: 'player-1', updated: false })],
        skipped: [],
      }),
    );
    expect(mocks.audit.record).toHaveBeenCalledWith(
      expect.objectContaining({ action: 'players_imported', entityId: 'roster.csv' }),
    );
  });

  it('returns unresolved name-duplicate rows before importing', async () => {
    mocks.players.findAll.mockResolvedValue([
      { firstName: 'Casey', id: 'existing-player', lastName: 'Jones' },
    ]);
    mocks.memberships.findAll.mockResolvedValue([
      {
        participationType: 'trains_and_plays',
        playerId: 'existing-player',
        relationship: 'primary',
        seasonStartingYear: 2026,
        status: 'active',
        teamName: 'U16-1',
      },
    ]);

    const response = await POST({
      request: new Request('http://localhost/api/imports/players', {
        body: JSON.stringify({
          content: 'first,last\nCasey,Jones',
          encoding: 'text',
          fileName: 'roster.csv',
          mapping: { firstName: 'first', lastName: 'last' },
          sourceName: 'roster.csv',
        }),
        headers: { 'content-type': 'application/json' },
        method: 'POST',
      }),
    } as never);

    expect(response.status).toBe(409);
    await expect(response.json()).resolves.toMatchObject({
      duplicates: [
        {
          candidates: [{ firstName: 'Casey', id: 'existing-player', lastName: 'Jones' }],
          sourceRow: 2,
        },
      ],
      error: 'import_duplicates',
    });
    expect(mocks.players.save).not.toHaveBeenCalled();
  });

  it('applies a supplied overwrite resolution for a flagged duplicate', async () => {
    mocks.players.findAll.mockResolvedValue([
      { firstName: 'Casey', id: 'existing-player', lastName: 'Jones' },
    ]);
    mocks.memberships.findAll.mockResolvedValue([
      {
        participationType: 'trains_and_plays',
        playerId: 'existing-player',
        relationship: 'primary',
        seasonStartingYear: 2026,
        status: 'active',
        teamName: 'U16-1',
      },
    ]);
    mocks.players.findById.mockResolvedValue({
      firstName: 'Casey',
      id: 'existing-player',
      lastName: 'Jones',
    });
    mocks.players.update.mockImplementation(async (player) => player);

    const response = await POST({
      request: new Request('http://localhost/api/imports/players', {
        body: JSON.stringify({
          content: 'first,last\nCasey,Jones',
          encoding: 'text',
          fileName: 'roster.csv',
          mapping: { firstName: 'first', lastName: 'last' },
          resolutions: [{ choice: 'overwrite', matchedPlayerId: 'existing-player', sourceRow: 2 }],
          sourceName: 'roster.csv',
        }),
        headers: { 'content-type': 'application/json' },
        method: 'POST',
      }),
    } as never);

    expect(response.status).toBe(200);
    await expect(response.json()).resolves.toEqual(
      expect.objectContaining({
        imported: [expect.objectContaining({ playerId: 'existing-player', updated: true })],
      }),
    );
    expect(mocks.players.save).not.toHaveBeenCalled();
  });
});
