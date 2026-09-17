import { describe, expect, it } from 'vitest';

import { InMemoryPlayerRepository } from '../../adapters/in-memory-player-repository';
import { configurePlayer } from './configure-player';

describe('configure player', () => {
  it('stores a player with the required core details', async () => {
    const players = new InMemoryPlayerRepository();

    const player = await configurePlayer(players, {
      associationId: '12345',
      birthDate: '2011-06-15',
      name: 'Avery',
    });

    expect(player).toEqual({
      associationId: '12345',
      birthDate: '2011-06-15',
      name: 'Avery',
    });
    await expect(players.findByAssociationId('12345')).resolves.toEqual(player);
  });

  it('returns an already-configured player rather than creating a duplicate', async () => {
    const players = new InMemoryPlayerRepository([
      { associationId: '12345', birthDate: '2011-06-15', name: 'Avery' },
    ]);

    await expect(
      configurePlayer(players, {
        associationId: '12345',
        birthDate: '2011-06-15',
        name: 'Avery',
      }),
    ).resolves.toEqual({ associationId: '12345', birthDate: '2011-06-15', name: 'Avery' });
  });
});
