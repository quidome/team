import { describe, expect, it } from 'vitest';

import { InMemoryClubRepository } from '../../adapters/in-memory-club-repository';
import { InMemoryTeamRepository } from '../../adapters/in-memory-team-repository';
import { assignTeamToClub, configureOwnClub } from './configure-club';

describe('configure club', () => {
  it('stores the own club with its source club ID', async () => {
    const clubs = new InMemoryClubRepository();

    const club = await configureOwnClub(clubs, { name: 'Blue Drakes', sourceClubId: 37 });

    expect(club).toEqual({ isOwnClub: true, name: 'Blue Drakes', sourceClubId: 37 });
    await expect(clubs.findOwnClub()).resolves.toEqual(club);
    await expect(clubs.findBySourceClubId(37)).resolves.toEqual(club);
  });

  it('keeps the known source club ID when it is not given again', async () => {
    const clubs = new InMemoryClubRepository([
      { isOwnClub: true, name: 'Blue Drakes', sourceClubId: 37 },
    ]);

    await expect(configureOwnClub(clubs, { name: 'Blue Drakes' })).resolves.toEqual({
      isOwnClub: true,
      name: 'Blue Drakes',
      sourceClubId: 37,
    });
  });

  it('rejects a second own club', async () => {
    const clubs = new InMemoryClubRepository([{ isOwnClub: true, name: 'Blue Drakes' }]);

    await expect(configureOwnClub(clubs, { name: 'Woodpeckers' })).rejects.toThrow(
      'Own club is already configured as Blue Drakes',
    );
  });

  it('rejects a source club ID that belongs to another club', async () => {
    const clubs = new InMemoryClubRepository([
      { isOwnClub: false, name: 'Woodpeckers', sourceClubId: 31 },
    ]);

    await expect(
      configureOwnClub(clubs, { name: 'Blue Drakes', sourceClubId: 31 }),
    ).rejects.toThrow('Source club ID 31 belongs to Woodpeckers');
  });

  it('assigns an existing team to a club', async () => {
    const clubs = new InMemoryClubRepository([{ isOwnClub: true, name: 'Blue Drakes' }]);
    const teams = new InMemoryTeamRepository([{ isOwnTeam: true, name: 'U16-1' }]);

    await assignTeamToClub(clubs, teams, 'Blue Drakes', 'U16-1');

    await expect(clubs.findTeamNames('Blue Drakes')).resolves.toEqual(['U16-1']);
  });

  it('rejects assigning a team that does not exist', async () => {
    const clubs = new InMemoryClubRepository([{ isOwnClub: true, name: 'Blue Drakes' }]);

    await expect(
      assignTeamToClub(clubs, new InMemoryTeamRepository(), 'Blue Drakes', 'U16-1'),
    ).rejects.toThrow('Team U16-1 does not exist');
  });
});
