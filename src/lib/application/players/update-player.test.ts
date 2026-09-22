import { describe, expect, it } from 'vitest';

import { InMemoryPlayerRepository } from '../../adapters/in-memory-player-repository';
import { updatePlayer } from './update-player';

describe('update player', () => {
  it('updates an existing player', async () => {
    const players = new InMemoryPlayerRepository([{ id: 'player-1', firstName: 'Avery' }]);

    const updated = await updatePlayer(players, {
      id: 'player-1',
      associationId: '12345',
      birthDate: '2011-06-15',
      firstName: 'Avery',
      lastName: 'Smith',
    });

    expect(updated).toEqual({
      id: 'player-1',
      associationId: '12345',
      birthDate: '2011-06-15',
      firstName: 'Avery',
      lastName: 'Smith',
    });
    await expect(players.findById('player-1')).resolves.toEqual(updated);
  });

  it('rejects an association ID already used by a different player', async () => {
    const players = new InMemoryPlayerRepository([
      { id: 'player-1', associationId: '12345', firstName: 'Avery' },
      { id: 'player-2', firstName: 'Blake' },
    ]);

    await expect(
      updatePlayer(players, { id: 'player-2', associationId: '12345', firstName: 'Blake' }),
    ).rejects.toThrow('Association ID 12345 is already in use');
  });

  it('allows a player to keep their own association ID', async () => {
    const players = new InMemoryPlayerRepository([
      { id: 'player-1', associationId: '12345', firstName: 'Avery' },
    ]);

    await expect(
      updatePlayer(players, { id: 'player-1', associationId: '12345', firstName: 'Avery Jones' }),
    ).resolves.toEqual({ id: 'player-1', associationId: '12345', firstName: 'Avery Jones' });
  });

  it('rejects updating a nonexistent player', async () => {
    const players = new InMemoryPlayerRepository();

    await expect(updatePlayer(players, { id: 'missing', firstName: 'Avery' })).rejects.toThrow(
      'Player does not exist',
    );
  });
});
