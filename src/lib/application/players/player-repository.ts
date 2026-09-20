export interface Player {
  associationId: string;
  birthDate: string;
  name: string;
}

export interface PlayerRepository {
  findAll(): Promise<Player[]>;
  findByAssociationId(associationId: string): Promise<Player | undefined>;
  save(player: Player): Promise<Player>;
  deleteByAssociationId(associationId: string): Promise<void>;
}
