import { describe, expect, it } from 'vitest';

import { InMemoryCoordinatorSettingsRepository } from '../../adapters/in-memory-coordinator-settings-repository';
import { InMemorySeasonRepository } from '../../adapters/in-memory-season-repository';
import { InMemoryTeamRepository } from '../../adapters/in-memory-team-repository';
import { configureCoordinatorSettings } from './configure-coordinator-settings';

describe('configure coordinator settings', () => {
  it('stores the primary team and season once both exist', async () => {
    const settings = new InMemoryCoordinatorSettingsRepository();
    const teams = new InMemoryTeamRepository([{ name: 'Blue Drakes M16-1' }]);
    const seasons = new InMemorySeasonRepository([{ endingYear: 2027, startingYear: 2026 }]);

    const result = await configureCoordinatorSettings(settings, teams, seasons, {
      primaryTeamName: 'Blue Drakes M16-1',
      seasonStartingYear: 2026,
    });

    expect(result).toEqual({ primaryTeamName: 'Blue Drakes M16-1', seasonStartingYear: 2026 });
    await expect(settings.get()).resolves.toEqual(result);
  });

  it('rejects a team that does not exist', async () => {
    const settings = new InMemoryCoordinatorSettingsRepository();
    const teams = new InMemoryTeamRepository();
    const seasons = new InMemorySeasonRepository([{ endingYear: 2027, startingYear: 2026 }]);

    await expect(
      configureCoordinatorSettings(settings, teams, seasons, {
        primaryTeamName: 'Blue Drakes M16-1',
        seasonStartingYear: 2026,
      }),
    ).rejects.toThrow('Team Blue Drakes M16-1 does not exist');
  });

  it('rejects a season that does not exist', async () => {
    const settings = new InMemoryCoordinatorSettingsRepository();
    const teams = new InMemoryTeamRepository([{ name: 'Blue Drakes M16-1' }]);
    const seasons = new InMemorySeasonRepository();

    await expect(
      configureCoordinatorSettings(settings, teams, seasons, {
        primaryTeamName: 'Blue Drakes M16-1',
        seasonStartingYear: 2026,
      }),
    ).rejects.toThrow('Season 2026 does not exist');
  });
});
