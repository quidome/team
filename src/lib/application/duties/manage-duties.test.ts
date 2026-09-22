import { describe, expect, it } from 'vitest';

import { InMemoryDutyRepository } from '../../adapters/in-memory-duty-repository';
import { InMemoryGameRepository } from '../../adapters/in-memory-game-repository';
import { configureGameFixture, scheduleGameOccurrence } from '../games/configure-game';
import { assignDuty, completeDueDuties, configureDuties, recordDutySignup } from './manage-duties';

describe('manage duties', () => {
  it('retains multiple signups while only the final assignment earns completion credit', async () => {
    const games = new InMemoryGameRepository();
    const duties = new InMemoryDutyRepository();
    const fixture = await configureGameFixture(games, {
      awayTeamName: 'U18-1',
      homeTeamName: 'U16-1',
    });
    const occurrence = await scheduleGameOccurrence(games, fixture.id, {
      arrivalBufferMinutes: 30,
      date: '2026-08-15',
      locationName: 'Away court',
      startTime: '14:30',
      travelMinutes: 20,
    });
    const configured = await configureDuties(duties, occurrence.id, {
      drivingSlots: 1,
      jurySlots: 0,
      refereeSlots: 1,
    });

    await recordDutySignup(duties, {
      dutyType: 'referee',
      occurrenceId: occurrence.id,
      playerId: 'avery',
      status: 'volunteer',
    });
    await recordDutySignup(duties, {
      dutyType: 'referee',
      occurrenceId: occurrence.id,
      playerId: 'blake',
      status: 'volunteer',
    });
    await assignDuty(
      duties,
      configured.slots.find((slot) => slot.dutyType === 'referee')!.id,
      'avery',
    );
    const reassigned = await assignDuty(
      duties,
      configured.slots.find((slot) => slot.dutyType === 'referee')!.id,
      'blake',
    );
    await completeDueDuties(duties, games, new Date('2026-08-16T00:00:00.000Z'));

    expect(reassigned.signups).toEqual([
      expect.objectContaining({ playerId: 'avery', status: 'volunteer' }),
      expect.objectContaining({ playerId: 'blake', status: 'selected' }),
    ]);
    await expect(duties.findByOccurrence(occurrence.id)).resolves.toEqual(
      expect.objectContaining({
        fairness: [{ completedCount: 1, playerId: 'blake' }],
        slots: expect.arrayContaining([
          expect.objectContaining({
            assignedPlayerId: 'blake',
            status: 'completed',
          }),
        ]),
      }),
    );
    await expect(duties.findByOccurrence(occurrence.id)).resolves.toMatchObject({
      assignmentHistory: expect.arrayContaining([
        expect.objectContaining({ playerId: 'avery', status: 'reassigned' }),
        expect.objectContaining({ playerId: 'blake', status: 'completed' }),
      ]),
    });
  });
});
