export interface Player {
  associationId: string;
  birthDate: string;
  name: string;
}

export interface PlayerRepository {
  findByAssociationId(associationId: string): Promise<Player | undefined>;
  save(player: Player): Promise<Player>;
}
