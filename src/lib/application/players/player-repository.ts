export interface Player {
  id: string;
  associationId?: string;
  birthDate?: string;
  firstName: string;
  lastName?: string;
}

export interface PlayerRepository {
  findAll(): Promise<Player[]>;
  findById(id: string): Promise<Player | undefined>;
  findByAssociationId(associationId: string): Promise<Player | undefined>;
  save(player: Omit<Player, 'id'>): Promise<Player>;
  update(player: Player): Promise<Player>;
  deleteById(id: string): Promise<void>;
}
