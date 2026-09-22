import type { DutyFairness } from '../duties/duty-repository';
import type { ParticipationRecord } from '../participation/participation-repository';
import type { ProgramEvent } from '../program/read-program';
import type { TeamPlayer } from '../team/read-team';

export type HistoryDenominator = 'recorded' | 'scheduled';

export interface HistoryMetric {
  denominator: number;
  percentage?: number;
  present: number;
  recorded: number;
}

export interface HistoryPlayerSummary {
  dutiesCompleted: number;
  firstName: string;
  games: HistoryMetric;
  id: string;
  lastName?: string;
  trainings: HistoryMetric;
}

export interface HistoryParticipationEntry {
  date?: string;
  eventId: string;
  eventLabel: string;
  occurrenceType: ParticipationRecord['occurrenceType'];
  playerId: string;
  status: ParticipationRecord['status'];
}

export interface HistoryReport {
  denominator: HistoryDenominator;
  entries: HistoryParticipationEntry[];
  players: HistoryPlayerSummary[];
}

const metric = (
  records: ParticipationRecord[],
  denominator: HistoryDenominator,
  scheduledCount: number,
): HistoryMetric => {
  const present = records.filter((record) => record.status === 'present').length;
  const recorded = records.length;
  const denominatorCount = denominator === 'scheduled' ? scheduledCount : recorded;

  return {
    denominator: denominatorCount,
    ...(denominatorCount > 0 ? { percentage: Math.round((present / denominatorCount) * 100) } : {}),
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
  denominator: HistoryDenominator = 'recorded',
): HistoryReport => {
  const teamPlayerIds = new Set(
    players
      .filter(
        (player) =>
          player.membership?.status === 'active' && player.membership.teamName === teamName,
      )
      .map((player) => player.id),
  );
  const eventsById = new Map(
    events.map((event) => [`${event.type}:${eventId(event)}`, event] as const),
  );
  const scheduledEvents = events.filter(
    (event) =>
      (event.type === 'training' ||
        event.homeTeamName === teamName ||
        event.awayTeamName === teamName) &&
      event.status === 'scheduled',
  );
  const teamRecords = records.filter((record) => teamPlayerIds.has(record.playerId));
  const fairnessByPlayer = new Map(fairness.map((entry) => [entry.playerId, entry.completedCount]));
  const playersById = new Map(players.map((player) => [player.id, player]));
  const summaries = players
    .filter((player) => teamPlayerIds.has(player.id))
    .map((player) => {
      const playerRecords = teamRecords.filter((record) => record.playerId === player.id);

      const playerRecordsFor = (occurrenceType: ParticipationRecord['occurrenceType']) =>
        playerRecords.filter((record) => {
          if (record.occurrenceType !== occurrenceType) {
            return false;
          }

          return denominator === 'recorded'
            ? true
            : scheduledEvents.some(
                (event) =>
                  `${event.type}:${eventId(event)}` ===
                  `${record.occurrenceType}:${record.occurrenceId}`,
              );
        });
      const scheduledCountFor = (occurrenceType: ParticipationRecord['occurrenceType']) =>
        scheduledEvents.filter(
          (event) =>
            event.type === occurrenceType &&
            (occurrenceType === 'training' ||
              player.membership?.participationType === 'trains_and_plays'),
        ).length;
      const gameRecords = playerRecordsFor('game');
      const trainingRecords = playerRecordsFor('training');

      return {
        dutiesCompleted: fairnessByPlayer.get(player.id) ?? 0,
        firstName: player.firstName,
        games: metric(gameRecords, denominator, scheduledCountFor('game')),
        id: player.id,
        ...(player.lastName ? { lastName: player.lastName } : {}),
        trainings: metric(trainingRecords, denominator, scheduledCountFor('training')),
      };
    })
    .sort((left, right) => left.firstName.localeCompare(right.firstName));
  const entries = teamRecords
    .map((record) => {
      const event = eventsById.get(`${record.occurrenceType}:${record.occurrenceId}`);

      return {
        ...(event ? { date: event.date } : {}),
        eventId: record.occurrenceId,
        eventLabel: event ? eventLabel(event) : record.occurrenceId,
        occurrenceType: record.occurrenceType,
        playerId: record.playerId,
        status: record.status,
      };
    })
    .sort((left, right) => (right.date ?? '').localeCompare(left.date ?? ''));

  return {
    denominator,
    entries: entries.filter((entry) => playersById.has(entry.playerId)),
    players: summaries,
  };
};
