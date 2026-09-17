export interface Season {
  endingYear: number;
  startingYear: number;
}

export interface SeasonRepository {
  findByStartingYear(startingYear: number): Promise<Season | undefined>;
  save(season: Season): Promise<Season>;
}
