import type { DutyFairness } from '../duties/duty-repository';
import type { ParticipationRecord } from '../participation/participation-repository';
import type { ProgramEvent } from '../program/read-program';
import type { TeamPlayer } from '../team/read-team';

export interface HistoryMetric {
  percentage?: number;
  present: number;
  recorded: number;
}

export interface HistoryPlayerSummary {
  associationId: string;
  dutiesCompleted: number;
  games: HistoryMetric;
  name: string;
  trainings: HistoryMetric;
}

export interface HistoryParticipationEntry {
  date?: string;
  eventId: string;
  eventLabel: string;
  occurrenceType: ParticipationRecord['occurrenceType'];
  playerAssociationId: string;
  status: ParticipationRecord['status'];
}

export interface HistoryReport {
  entries: HistoryParticipationEntry[];
  players: HistoryPlayerSummary[];
}

const metric = (records: ParticipationRecord[]): HistoryMetric => {
  const present = records.filter((record) => record.status === 'present').length;
  const recorded = records.length;

  return {
    ...(recorded > 0 ? { percentage: Math.round((present / recorded) * 100) } : {}),
    present,
    recorded,
  };
};

const eventId = (event: ProgramEvent) =>
  event.type === 'game' ? event.id : (event.occurrenceId ?? event.id);

const eventLabel = (event: ProgramEvent) =>
  event.type === 'game' ? `${event.homeTeamName} vs ${event.awayTeamName}` : 'Team training';

export const readHistory = (
  players: TeamPlayer[],
  records: ParticipationRecord[],
  fairness: DutyFairness[],
  events: ProgramEvent[],
  teamName = 'U16-1',
): HistoryReport => {
  const teamPlayerIds = new Set(
    players
      .filter(
        (player) =>
          player.membership?.status === 'active' && player.membership.teamName === teamName,
      )
      .map((player) => player.associationId),
  );
  const eventsById = new Map(events.map((event) => [eventId(event), event]));
  const teamRecords = records.filter((record) => teamPlayerIds.has(record.playerAssociationId));
  const fairnessByPlayer = new Map(
    fairness.map((entry) => [entry.playerAssociationId, entry.completedCount]),
  );
  const playersById = new Map(players.map((player) => [player.associationId, player]));
  const summaries = players
    .filter((player) => teamPlayerIds.has(player.associationId))
    .map((player) => {
      const playerRecords = teamRecords.filter(
        (record) => record.playerAssociationId === player.associationId,
      );

      return {
        associationId: player.associationId,
        dutiesCompleted: fairnessByPlayer.get(player.associationId) ?? 0,
        games: metric(playerRecords.filter((record) => record.occurrenceType === 'game')),
        name: player.name,
        trainings: metric(playerRecords.filter((record) => record.occurrenceType === 'training')),
      };
    })
    .sort((left, right) => left.name.localeCompare(right.name));
  const entries = teamRecords
    .map((record) => {
      const event = eventsById.get(record.occurrenceId);

      return {
        ...(event ? { date: event.date } : {}),
        eventId: record.occurrenceId,
        eventLabel: event ? eventLabel(event) : record.occurrenceId,
        occurrenceType: record.occurrenceType,
        playerAssociationId: record.playerAssociationId,
        status: record.status,
      };
    })
    .sort((left, right) => (right.date ?? '').localeCompare(left.date ?? ''));

  return {
    entries: entries.filter((entry) => playersById.has(entry.playerAssociationId)),
    players: summaries,
  };
};
