import type { Player, PlayerRepository } from '../application/players/player-repository';

export class InMemoryPlayerRepository implements PlayerRepository {
  private readonly players = new Map<string, Player>();

  constructor(initialPlayers: Player[] = []) {
    for (const player of initialPlayers) {
      this.players.set(player.associationId, player);
    }
  }

  async findByAssociationId(associationId: string): Promise<Player | undefined> {
    return this.players.get(associationId);
  }

  async save(player: Player): Promise<Player> {
    this.players.set(player.associationId, player);

    return player;
  }
}
