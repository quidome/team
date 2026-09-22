import type { Player, PlayerRepository } from '../players/player-repository';
import type { MembershipRepository } from '../memberships/membership-repository';
import { configurePlayer } from '../players/configure-player';
import { updatePlayer } from '../players/update-player';
import { configureMembership } from '../memberships/configure-membership';
import type { ImportedPlayer } from './player-import';

export interface PlayerImportContext {
  primaryTeamName: string;
  seasonStartingYear: number;
}

export interface ImportedPlayerResult extends ImportedPlayer {
  playerId: string;
  updated: boolean;
}

export interface FailedPlayerResult {
  message: string;
  sourceRow: number;
}

export interface SkippedPlayerResult {
  message: string;
  sourceRow: number;
}

export interface PlayerImportResult {
  failed: FailedPlayerResult[];
  imported: ImportedPlayerResult[];
  skipped: SkippedPlayerResult[];
}

export type PlayerImportChoice = 'add' | 'overwrite' | 'skip';

export interface PlayerImportDuplicateCandidate {
  associationId?: string;
  birthDate?: string;
  firstName: string;
  id: string;
  lastName?: string;
}

export interface PlayerImportDuplicate {
  candidates: PlayerImportDuplicateCandidate[];
  sourceRow: number;
}

export interface PlayerImportResolution {
  choice: PlayerImportChoice;
  matchedPlayerId?: string;
  sourceRow: number;
}

export interface ImportPlayersInput {
  context: PlayerImportContext;
  records: ImportedPlayer[];
  resolutions?: PlayerImportResolution[];
}

const identityKey = (firstName: string, lastName?: string): string =>
  `${firstName.trim().toLowerCase()}|${(lastName ?? '').trim().toLowerCase()}`;

export const findImportDuplicates = async (
  players: PlayerRepository,
  memberships: MembershipRepository,
  records: ImportedPlayer[],
  context: PlayerImportContext,
): Promise<{ duplicates: PlayerImportDuplicate[] }> => {
  const [allPlayers, allMemberships] = await Promise.all([
    players.findAll(),
    memberships.findAll(),
  ]);
  const rosterPlayerIds = new Set(
    allMemberships
      .filter(
        (membership) =>
          membership.teamName === context.primaryTeamName &&
          membership.seasonStartingYear === context.seasonStartingYear,
      )
      .map((membership) => membership.playerId),
  );
  const candidatesByKey = new Map<string, PlayerImportDuplicateCandidate[]>();

  for (const player of allPlayers) {
    if (!rosterPlayerIds.has(player.id)) {
      continue;
    }

    const key = identityKey(player.firstName, player.lastName);
    const candidate: PlayerImportDuplicateCandidate = {
      ...(player.associationId ? { associationId: player.associationId } : {}),
      ...(player.birthDate ? { birthDate: player.birthDate } : {}),
      firstName: player.firstName,
      id: player.id,
      ...(player.lastName ? { lastName: player.lastName } : {}),
    };
    const existingCandidates = candidatesByKey.get(key) ?? [];

    existingCandidates.push(candidate);
    candidatesByKey.set(key, existingCandidates);
  }

  const duplicates: PlayerImportDuplicate[] = [];

  for (const record of records) {
    if (record.associationId) {
      continue;
    }

    const candidates = candidatesByKey.get(identityKey(record.firstName, record.lastName));

    if (candidates && candidates.length > 0) {
      duplicates.push({ candidates, sourceRow: record.sourceRow });
    }
  }

  return { duplicates };
};

export const importPlayers = async (
  players: PlayerRepository,
  memberships: MembershipRepository,
  input: ImportPlayersInput,
): Promise<PlayerImportResult> => {
  const imported: ImportedPlayerResult[] = [];
  const skipped: SkippedPlayerResult[] = [];
  const failed: FailedPlayerResult[] = [];
  const resolutionsByRow = new Map(
    (input.resolutions ?? []).map((resolution) => [resolution.sourceRow, resolution]),
  );

  for (const record of input.records) {
    try {
      const resolution = record.associationId ? undefined : resolutionsByRow.get(record.sourceRow);

      if (resolution?.choice === 'skip') {
        skipped.push({
          message: 'The coordinator chose to skip this row.',
          sourceRow: record.sourceRow,
        });
        continue;
      }

      const fields = {
        ...(record.associationId ? { associationId: record.associationId } : {}),
        ...(record.birthDate ? { birthDate: record.birthDate } : {}),
        firstName: record.firstName,
        ...(record.lastName ? { lastName: record.lastName } : {}),
      };

      let player: Player;
      let wasUpdate = false;

      if (record.associationId) {
        const existing = await players.findByAssociationId(record.associationId);

        if (existing) {
          player = await updatePlayer(players, { ...existing, ...fields, id: existing.id });
          wasUpdate = true;
        } else {
          player = await configurePlayer(players, fields);
        }
      } else if (resolution?.choice === 'overwrite' && resolution.matchedPlayerId) {
        const existing = await players.findById(resolution.matchedPlayerId);

        if (!existing) {
          throw new Error('The matched player no longer exists.');
        }

        player = await updatePlayer(players, { ...existing, ...fields, id: existing.id });
        wasUpdate = true;
      } else {
        player = await configurePlayer(players, fields);
      }

      await configureMembership(memberships, {
        ...(record.jerseyNumber !== undefined ? { jerseyNumber: record.jerseyNumber } : {}),
        participationType: record.participationType,
        playerId: player.id,
        relationship: 'primary',
        seasonStartingYear: input.context.seasonStartingYear,
        status: 'active',
        teamName: input.context.primaryTeamName,
      });

      imported.push({ ...record, playerId: player.id, updated: wasUpdate });
    } catch (error) {
      failed.push({
        message: error instanceof Error ? error.message : 'The player could not be imported.',
        sourceRow: record.sourceRow,
      });
    }
  }

  return { failed, imported, skipped };
};
