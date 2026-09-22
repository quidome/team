import { describe, expect, it } from 'vitest';

import { InMemoryMembershipRepository } from '../../adapters/in-memory-membership-repository';
import { InMemoryPlayerRepository } from '../../adapters/in-memory-player-repository';
import { importPlayers } from './import-players';

const context = { primaryTeamName: 'U16-1', seasonStartingYear: 2026 };

describe('import players', () => {
  it('creates a new player and membership for a row with an association ID', async () => {
    const players = new InMemoryPlayerRepository();
    const memberships = new InMemoryMembershipRepository();

    const result = await importPlayers(players, memberships, {
      context,
      records: [
        {
          associationId: '12345',
          birthDate: '2011-06-15',
          firstName: 'Avery',
          jerseyNumber: 7,
          participationType: 'trains_and_plays',
          sourceRow: 2,
        },
      ],
    });

    expect(result.failed).toEqual([]);
    expect(result.imported).toHaveLength(1);
    expect(result.imported[0]).toMatchObject({ associationId: '12345', updated: false });

    const storedPlayers = await players.findAll();
    expect(storedPlayers).toHaveLength(1);
    expect(storedPlayers[0]).toMatchObject({ associationId: '12345', firstName: 'Avery' });

    const storedMemberships = await memberships.findAll();
    expect(storedMemberships).toEqual([
      {
        jerseyNumber: 7,
        participationType: 'trains_and_plays',
        playerId: storedPlayers[0].id,
        relationship: 'primary',
        seasonStartingYear: 2026,
        status: 'active',
        teamName: 'U16-1',
      },
    ]);
  });

  it('updates an existing player matched by association ID and refreshes the membership', async () => {
    const players = new InMemoryPlayerRepository();
    const existing = await players.save({
      associationId: '12345',
      birthDate: '2010-01-01',
      firstName: 'Old Name',
    });
    const memberships = new InMemoryMembershipRepository([
      {
        jerseyNumber: 4,
        participationType: 'trains_only',
        playerId: existing.id,
        relationship: 'primary',
        seasonStartingYear: 2026,
        status: 'active',
        teamName: 'U16-1',
      },
    ]);

    const result = await importPlayers(players, memberships, {
      context,
      records: [
        {
          associationId: '12345',
          birthDate: '2011-06-15',
          firstName: 'Avery',
          jerseyNumber: 9,
          participationType: 'trains_and_plays',
          sourceRow: 2,
        },
      ],
    });

    expect(result.failed).toEqual([]);
    expect(result.imported[0]).toMatchObject({ playerId: existing.id, updated: true });
    await expect(players.findById(existing.id)).resolves.toEqual({
      associationId: '12345',
      birthDate: '2011-06-15',
      firstName: 'Avery',
      id: existing.id,
    });
    await expect(memberships.findAll()).resolves.toEqual([
      {
        jerseyNumber: 9,
        participationType: 'trains_and_plays',
        playerId: existing.id,
        relationship: 'primary',
        seasonStartingYear: 2026,
        status: 'active',
        teamName: 'U16-1',
      },
    ]);
  });

  it('always creates a new player for a row without an association ID', async () => {
    const players = new InMemoryPlayerRepository();
    await players.save({ firstName: 'Casey' });
    const memberships = new InMemoryMembershipRepository();

    const result = await importPlayers(players, memberships, {
      context,
      records: [{ firstName: 'Casey', participationType: 'trains_and_plays', sourceRow: 2 }],
    });

    expect(result.failed).toEqual([]);
    await expect(players.findAll()).resolves.toHaveLength(2);
  });

  it('records a per-row failure without aborting the rest of the import', async () => {
    const players = new InMemoryPlayerRepository();
    const brokenMemberships = {
      find: async () => undefined,
      findAll: async () => [],
      save: async () => {
        throw new Error('membership storage failed');
      },
      update: async () => {
        throw new Error('membership storage failed');
      },
    };

    const result = await importPlayers(players, brokenMemberships, {
      context,
      records: [
        { firstName: 'Avery', participationType: 'trains_and_plays', sourceRow: 2 },
        { firstName: 'Blake', participationType: 'trains_and_plays', sourceRow: 3 },
      ],
    });

    expect(result.imported).toEqual([]);
    expect(result.failed).toEqual([
      { message: 'membership storage failed', sourceRow: 2 },
      { message: 'membership storage failed', sourceRow: 3 },
    ]);
  });
});
