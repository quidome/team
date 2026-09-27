import { eq, inArray } from 'drizzle-orm';
import { afterAll, beforeAll, describe, expect, it } from 'vitest';

import type { ClubRepository } from '../../application/clubs/club-repository';
import { createDatabase } from './database';
import { createPostgresClubRepository } from './postgres-club-repository';
import { clubs, teams } from './schema';

const databaseUrl = process.env.DATABASE_URL;

if (!databaseUrl) {
  describe.skip('PostgreSQL club repository', () => {});
} else {
  describe('PostgreSQL club repository', () => {
    const clubName = 'Club integration test';
    const teamName = 'U16-1 club integration test';
    const sourceClubId = 987_654;
    const database = createDatabase(databaseUrl);
    const repository: ClubRepository = createPostgresClubRepository(database);

    const cleanUp = async () => {
      await database.delete(teams).where(eq(teams.name, teamName));
      await database.delete(clubs).where(inArray(clubs.name, [clubName]));
    };

    beforeAll(async () => {
      await cleanUp();
    });

    afterAll(async () => {
      await cleanUp();
      await database.close();
    });

    it('persists a club and finds it by name and source club ID', async () => {
      await expect(
        repository.save({ isOwnClub: false, name: clubName, sourceClubId }),
      ).resolves.toEqual({ isOwnClub: false, name: clubName, sourceClubId });
      await expect(repository.findByName(clubName)).resolves.toEqual({
        isOwnClub: false,
        name: clubName,
        sourceClubId,
      });
      await expect(repository.findBySourceClubId(sourceClubId)).resolves.toEqual({
        isOwnClub: false,
        name: clubName,
        sourceClubId,
      });
    });

    it('updates an existing club by name', async () => {
      await expect(repository.save({ isOwnClub: false, name: clubName })).resolves.toEqual({
        isOwnClub: false,
        name: clubName,
      });
    });

    it('assigns a team to the club', async () => {
      await database.insert(teams).values({ isOwnTeam: true, name: teamName });

      await repository.assignTeam(clubName, teamName);

      await expect(repository.findTeamNames(clubName)).resolves.toEqual([teamName]);
    });
  });
}
