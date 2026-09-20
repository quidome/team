import { defaultPollTemplates } from '$lib/application/messages/poll-drafts';
import { currentTeamName } from '$lib/server/load-team';
import { loadProgram } from './load-program';
import type { ProgramGameEvent } from '$lib/application/program/read-program';

export interface MessagesPageData {
  events: ProgramGameEvent[];
  teamName: string;
  templates: typeof defaultPollTemplates;
}

export const loadMessages = async (): Promise<MessagesPageData> => {
  const program = await loadProgram();

  return {
    events: program.events.filter(
      (event): event is ProgramGameEvent =>
        event.type === 'game' &&
        (event.homeTeamName === currentTeamName || event.awayTeamName === currentTeamName),
    ),
    teamName: currentTeamName,
    templates: defaultPollTemplates,
  };
};
