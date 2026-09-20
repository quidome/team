import { describe, expect, it } from 'vitest';

import { InMemoryTaskRepository } from '../../adapters/in-memory-task-repository';
import { readTasks } from './generate-tasks';

const game = {
  arrivalBufferMinutes: 30,
  awayTeamName: 'U18-1',
  date: '2026-08-15',
  fixtureId: 'fixture-1',
  homeTeamName: 'U16-1',
  id: 'game-1',
  locationName: 'Home court',
  startTime: '14:30',
  status: 'scheduled' as const,
  suggestedDepartureTime: '13:40',
  travelMinutes: 20,
  type: 'game' as const,
};

describe('generated tasks', () => {
  it('combines future generated coordination tasks and manual reminders', async () => {
    const tasks = new InMemoryTaskRepository([
      {
        dueDate: '2026-08-10',
        id: 'manual-1',
        source: 'manual',
        status: 'open',
        title: 'Confirm the roster',
      },
    ]);

    await expect(
      readTasks(tasks, [game], { teamName: 'U16-1', today: '2026-08-01' }),
    ).resolves.toEqual([
      expect.objectContaining({ source: 'manual', title: 'Confirm the roster' }),
      expect.objectContaining({
        description: 'Prepare the home-game attendance poll draft.',
        source: 'generated',
        title: 'Coordinate U16-1 vs U18-1',
      }),
    ]);
  });
});
