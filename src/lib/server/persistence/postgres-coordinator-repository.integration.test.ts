import { eq, inArray } from 'drizzle-orm';
import { afterAll, beforeAll, describe, expect, it } from 'vitest';

import type { CoordinatorRepository } from '../../application/coordinators/coordinator-repository';
import { createDatabase } from './database';
import { createPostgresCoordinatorRepository } from './postgres-coordinator-repository';
import { coordinators, teams } from './schema';

const databaseUrl = process.env.DATABASE_URL;

if (!databaseUrl) {
  describe.skip('PostgreSQL coordinator repository', () => {});
} else {
  describe('PostgreSQL coordinator repository', () => {
    const sam = 'coordinator integration test sam';
    const robin = 'coordinator integration test robin';
    const firstTeam = 'U16-1 coordinator integration test';
    const secondTeam = 'U16-2 coordinator integration test';
    const database = createDatabase(databaseUrl);
    const repository: CoordinatorRepository = createPostgresCoordinatorRepository(database);

    const cleanUp = async () => {
      await database.delete(coordinators).where(inArray(coordinators.subject, [sam, robin]));
      await database.delete(teams).where(inArray(teams.name, [firstTeam, secondTeam]));
    };

    beforeAll(async () => {
      await cleanUp();
      await database.insert(teams).values([
        { isOwnTeam: true, name: firstTeam },
        { isOwnTeam: true, name: secondTeam },
      ]);
    });

    afterAll(async () => {
      await cleanUp();
      await database.close();
    });

    it('stores a coordinator once per subject', async () => {
      await expect(repository.ensure(sam)).resolves.toEqual({ subject: sam, teamNames: [] });
      await repository.ensure(sam);

      const rows = await database
        .select({ id: coordinators.id })
        .from(coordinators)
        .where(eq(coordinators.subject, sam));

      expect(rows).toHaveLength(1);
    });

    it('links teams to coordinators in both directions', async () => {
      await repository.addTeam(sam, firstTeam);
      await repository.addTeam(sam, firstTeam);
      await repository.addTeam(sam, secondTeam);
      await repository.addTeam(robin, firstTeam);

      await expect(repository.findBySubject(sam)).resolves.toEqual({
        subject: sam,
        teamNames: [firstTeam, secondTeam],
      });
      await expect(repository.findSubjectsByTeam(firstTeam)).resolves.toEqual([robin, sam]);
    });

    it('sets, clears, and removes teams', async () => {
      await expect(repository.setDefaultTeam(sam, firstTeam)).resolves.toEqual({
        defaultTeamName: firstTeam,
        subject: sam,
        teamNames: [firstTeam, secondTeam],
      });
      await expect(repository.setDefaultTeam(sam, undefined)).resolves.toEqual({
        subject: sam,
        teamNames: [firstTeam, secondTeam],
      });
      await expect(repository.removeTeam(sam, firstTeam)).resolves.toEqual({
        subject: sam,
        teamNames: [secondTeam],
      });
    });

    it('stores and clears the profile email', async () => {
      await expect(
        repository.setProfile(robin, { displayName: 'Robin', email: 'robin@example.test' }),
      ).resolves.toMatchObject({ profile: { displayName: 'Robin', email: 'robin@example.test' } });
      await expect(repository.setProfile(robin, { displayName: 'Robin' })).resolves.toMatchObject({
        profile: { displayName: 'Robin' },
      });
    });
  });
}
