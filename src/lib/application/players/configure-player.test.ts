import { describe, expect, it } from 'vitest';

import { InMemoryPlayerRepository } from '../../adapters/in-memory-player-repository';
import { configurePlayer } from './configure-player';

describe('configure player', () => {
  it('stores a player with the required core details', async () => {
    const players = new InMemoryPlayerRepository();

    const player = await configurePlayer(players, {
      associationId: '12345',
      birthDate: '2011-06-15',
      firstName: 'Avery',
    });

    expect(player).toEqual({
      id: expect.any(String),
      associationId: '12345',
      birthDate: '2011-06-15',
      firstName: 'Avery',
    });
    await expect(players.findByAssociationId('12345')).resolves.toEqual(player);
  });

  it('returns an already-configured player rather than creating a duplicate', async () => {
    const players = new InMemoryPlayerRepository([
      { id: 'player-1', associationId: '12345', birthDate: '2011-06-15', firstName: 'Avery' },
    ]);

    await expect(
      configurePlayer(players, {
        associationId: '12345',
        birthDate: '2011-06-15',
        firstName: 'Avery',
      }),
    ).resolves.toEqual({
      id: 'player-1',
      associationId: '12345',
      birthDate: '2011-06-15',
      firstName: 'Avery',
    });
  });

  it('stores a player with only a first name', async () => {
    const players = new InMemoryPlayerRepository();

    const player = await configurePlayer(players, { firstName: 'Avery' });

    expect(player).toEqual({ id: expect.any(String), firstName: 'Avery' });
  });

  it('does not deduplicate players registered without an association ID', async () => {
    const players = new InMemoryPlayerRepository();

    const first = await configurePlayer(players, { firstName: 'Avery' });
    const second = await configurePlayer(players, { firstName: 'Avery' });

    expect(first.id).not.toBe(second.id);
    await expect(players.findAll()).resolves.toHaveLength(2);
  });
});
