import { defaultPollTemplates } from '$lib/application/messages/poll-drafts';
import { currentTeamSeasonContext } from '$lib/server/load-team';
import { loadProgram } from './load-program';
import type { ProgramGameEvent } from '$lib/application/program/read-program';

export interface MessagesPageData {
  events: ProgramGameEvent[];
  teamName: string;
  templates: typeof defaultPollTemplates;
}

export const loadMessages = async (): Promise<MessagesPageData> => {
  const [program, { teamName }] = await Promise.all([loadProgram(), currentTeamSeasonContext()]);

  return {
    events: program.events.filter(
      (event): event is ProgramGameEvent =>
        event.type === 'game' &&
        (event.homeTeamName === teamName || event.awayTeamName === teamName),
    ),
    teamName,
    templates: defaultPollTemplates,
  };
};
