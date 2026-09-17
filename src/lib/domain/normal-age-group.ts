export type NormalAgeGroup = `U${number}`;

export const calculateNormalAgeGroup = (
  seasonStartingYear: number,
  birthYear: number,
): NormalAgeGroup => {
  const ageAtSeasonStart = seasonStartingYear - birthYear;
  const ageGroup = ageAtSeasonStart % 2 === 0 ? ageAtSeasonStart + 2 : ageAtSeasonStart + 1;

  return `U${ageGroup}`;
};
