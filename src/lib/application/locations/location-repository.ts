export interface Location {
  name: string;
  travelMinutes: number;
}

export interface LocationRepository {
  findByName(name: string): Promise<Location | undefined>;
  save(location: Location): Promise<Location>;
}
