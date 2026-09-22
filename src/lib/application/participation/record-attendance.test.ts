import { describe, expect, it } from 'vitest';

import { InMemoryParticipationRepository } from '../../adapters/in-memory-participation-repository';
import { recordAttendance } from './record-attendance';

describe('record attendance', () => {
  it('stores present players and an absence reason in one bulk operation', async () => {
    const participation = new InMemoryParticipationRepository();

    const records = await recordAttendance(participation, {
      absences: [{ playerId: 'blake', reason: 'illness' }],
      eligiblePlayerIds: ['avery', 'blake', 'casey'],
      occurrenceId: 'training-occurrence-1',
      occurrenceType: 'training',
    });

    expect(records).toEqual([
      {
        absenceReason: undefined,
        occurrenceId: 'training-occurrence-1',
        occurrenceType: 'training',
        playerId: 'avery',
        status: 'present',
      },
      {
        absenceReason: 'illness',
        occurrenceId: 'training-occurrence-1',
        occurrenceType: 'training',
        playerId: 'blake',
        status: 'absent',
      },
      {
        absenceReason: undefined,
        occurrenceId: 'training-occurrence-1',
        occurrenceType: 'training',
        playerId: 'casey',
        status: 'present',
      },
    ]);
    await expect(
      participation.findByOccurrence('training', 'training-occurrence-1'),
    ).resolves.toEqual(records);
  });

  it('rejects an absence for a player outside the eligible set', async () => {
    const participation = new InMemoryParticipationRepository();

    await expect(
      recordAttendance(participation, {
        absences: [{ playerId: 'unknown', reason: 'other' }],
        eligiblePlayerIds: ['avery'],
        occurrenceId: 'training-occurrence-1',
        occurrenceType: 'training',
      }),
    ).rejects.toThrow('unknown is not eligible for the occurrence');
  });
});
