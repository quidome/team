export const normalizeTeamName = (value: string): string =>
  value
    .replace(/\s[-‐-―]\s/g, ' ')
    .replace(/\s+/g, ' ')
    .trim();

export const teamNamesMatch = (a: string, b: string): boolean =>
  normalizeTeamName(a).toLowerCase() === normalizeTeamName(b).toLowerCase();
