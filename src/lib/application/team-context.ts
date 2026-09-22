export interface TeamSeasonContext {
  seasonStartingYear: number;
  teamName: string;
}

export const fallbackTeamSeasonContext: TeamSeasonContext = {
  seasonStartingYear: 2026,
  teamName: 'U16-1',
};

export const formatTeamSeasonContext = ({ seasonStartingYear, teamName }: TeamSeasonContext) =>
  `${String(seasonStartingYear).slice(-2)}–${String(seasonStartingYear + 1).slice(-2)} · ${teamName}`;
