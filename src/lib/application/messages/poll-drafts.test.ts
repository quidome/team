import { describe, expect, it } from 'vitest';

import type { ProgramGameEvent } from '../program/read-program';
import { generatePollDraft } from './poll-drafts';

const game: ProgramGameEvent = {
  arrivalBufferMinutes: 30,
  awayTeamName: 'U16-1',
  date: '2026-08-15',
  fixtureId: 'fixture-1',
  homeTeamName: 'U18-1',
  id: 'game-1',
  locationName: 'Away court',
  startTime: '14:30',
  status: 'scheduled',
  suggestedDepartureTime: '13:40',
  travelMinutes: 20,
  type: 'game',
};

describe('poll drafts', () => {
  it('generates an editable away-game attendance and transport draft', () => {
    expect(
      generatePollDraft(game, {
        departureLocation: 'Clubhouse',
        teamName: 'U16-1',
      }),
    ).toEqual({
      kind: 'away',
      text: expect.stringContaining('Vertrek om 13:40 vanaf Clubhouse.'),
    });
  });

  it('uses a per-game template and departure override', () => {
    expect(
      generatePollDraft(game, {
        departureTimeOverride: '12:55',
        teamName: 'U16-1',
        templates: { awayGame: '{{team}} · {{departureTime}} · {{opponent}}' },
      }),
    ).toEqual({ kind: 'away', text: 'U16-1 · 12:55 · U18-1' });
  });
});
