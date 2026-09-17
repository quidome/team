import type { Season, SeasonRepository } from './season-repository';

export const configureSeason = async (
  seasons: SeasonRepository,
  startingYear: number,
): Promise<Season> => {
  const existingSeason = await seasons.findByStartingYear(startingYear);

  if (existingSeason) {
    return existingSeason;
  }

  return seasons.save({
    endingYear: startingYear + 1,
    startingYear,
  });
};
