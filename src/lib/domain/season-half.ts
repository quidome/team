export type SeasonHalf = 'H1' | 'H2';

export const deriveSeasonHalf = (
  gameDate: string,
  seasonStartingYear: number,
): SeasonHalf | undefined => {
  const year = Number(gameDate.slice(0, 4));

  if (year === seasonStartingYear) {
    return 'H1';
  }

  if (year === seasonStartingYear + 1) {
    return 'H2';
  }

  return undefined;
};
