import type { Player, PlayerRepository } from './player-repository';

export const configurePlayer = async (
  players: PlayerRepository,
  player: Omit<Player, 'id'>,
): Promise<Player> => {
  if (player.associationId) {
    const existingPlayer = await players.findByAssociationId(player.associationId);

    if (existingPlayer) {
      return existingPlayer;
    }
  }

  return players.save(player);
};
