import { randomUUID } from 'node:crypto';

import type { Player, PlayerRepository } from '../application/players/player-repository';

export class InMemoryPlayerRepository implements PlayerRepository {
  private readonly players = new Map<string, Player>();

  constructor(initialPlayers: Player[] = []) {
    for (const player of initialPlayers) {
      this.players.set(player.id, player);
    }
  }

  async findAll(): Promise<Player[]> {
    return [...this.players.values()];
  }

  async findById(id: string): Promise<Player | undefined> {
    return this.players.get(id);
  }

  async findByAssociationId(associationId: string): Promise<Player | undefined> {
    return [...this.players.values()].find((player) => player.associationId === associationId);
  }

  async save(player: Omit<Player, 'id'>): Promise<Player> {
    const storedPlayer: Player = { ...player, id: randomUUID() };
    this.players.set(storedPlayer.id, storedPlayer);

    return storedPlayer;
  }

  async update(player: Player): Promise<Player> {
    if (!this.players.has(player.id)) {
      throw new Error('Player does not exist');
    }

    this.players.set(player.id, player);

    return player;
  }

  async deleteById(id: string): Promise<void> {
    this.players.delete(id);
  }
}
