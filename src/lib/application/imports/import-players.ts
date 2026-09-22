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

export interface PlayerImportResult {
  failed: FailedPlayerResult[];
  imported: ImportedPlayerResult[];
}

export interface ImportPlayersInput {
  context: PlayerImportContext;
  records: ImportedPlayer[];
}

export const importPlayers = async (
  players: PlayerRepository,
  memberships: MembershipRepository,
  input: ImportPlayersInput,
): Promise<PlayerImportResult> => {
  const imported: ImportedPlayerResult[] = [];
  const failed: FailedPlayerResult[] = [];

  for (const record of input.records) {
    try {
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

  return { failed, imported };
};
