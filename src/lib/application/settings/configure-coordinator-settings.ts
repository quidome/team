import type {
  CoordinatorSettings,
  CoordinatorSettingsRepository,
} from './coordinator-settings-repository';
import type { SeasonRepository } from '../seasons/season-repository';
import type { TeamRepository } from '../teams/team-repository';

export const configureCoordinatorSettings = async (
  settings: CoordinatorSettingsRepository,
  teams: TeamRepository,
  seasons: SeasonRepository,
  input: CoordinatorSettings,
): Promise<CoordinatorSettings> => {
  const [team, season] = await Promise.all([
    teams.findByName(input.primaryTeamName),
    seasons.findByStartingYear(input.seasonStartingYear),
  ]);

  if (!team) {
    throw new Error(`Team ${input.primaryTeamName} does not exist`);
  }

  if (!season) {
    throw new Error(`Season ${input.seasonStartingYear} does not exist`);
  }

  return settings.save(input);
};
