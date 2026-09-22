import type { Player, PlayerRepository } from './player-repository';

export const updatePlayer = async (players: PlayerRepository, player: Player): Promise<Player> => {
  if (player.associationId) {
    const existingPlayer = await players.findByAssociationId(player.associationId);

    if (existingPlayer && existingPlayer.id !== player.id) {
      throw new Error(`Association ID ${player.associationId} is already in use`);
    }
  }

  return players.update(player);
};
