import { eq } from 'drizzle-orm';

import type { Team, TeamRepository } from '../../application/teams/team-repository';
import { createDatabase } from './database';
import { teams } from './schema';

type Database = ReturnType<typeof createDatabase>;

export const createPostgresTeamRepository = (database: Database): TeamRepository => ({
  async findByName(name: string): Promise<Team | undefined> {
    const [team] = await database
      .select({ name: teams.name })
      .from(teams)
      .where(eq(teams.name, name))
      .limit(1);

    return team;
  },

  async save(team: Team): Promise<Team> {
    const [storedTeam] = await database.insert(teams).values(team).returning({ name: teams.name });

    if (!storedTeam) {
      throw new Error('PostgreSQL did not return the stored team');
    }

    return storedTeam;
  },
});
