export interface Location {
  address?: string;
  name: string;
  travelMinutes: number;
}

export interface LocationRepository {
  findAll(): Promise<Location[]>;
  findByName(name: string): Promise<Location | undefined>;
  save(location: Location): Promise<Location>;
  updateName(currentName: string, location: Location): Promise<Location>;
  deleteByName(name: string): Promise<void>;
}
