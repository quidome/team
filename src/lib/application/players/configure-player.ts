import type { Player, PlayerRepository } from './player-repository';

export const configurePlayer = async (
  players: PlayerRepository,
  player: Player,
): Promise<Player> => {
  const existingPlayer = await players.findByAssociationId(player.associationId);

  if (existingPlayer) {
    return existingPlayer;
  }

  return players.save(player);
};
