import type { Season, SeasonRepository } from '../application/seasons/season-repository';

export class InMemorySeasonRepository implements SeasonRepository {
  private readonly seasons = new Map<number, Season>();

  constructor(initialSeasons: Season[] = []) {
    for (const season of initialSeasons) {
      this.seasons.set(season.startingYear, season);
    }
  }

  async findByStartingYear(startingYear: number): Promise<Season | undefined> {
    return this.seasons.get(startingYear);
  }

  async save(season: Season): Promise<Season> {
    this.seasons.set(season.startingYear, season);

    return season;
  }
}
