import type { ProgramGameEvent } from '../program/read-program';

export type PollDraftKind = 'away' | 'home';

export interface PollTemplates {
  awayGame: string;
  homeGame: string;
}

export interface PollDraftOptions {
  departureLocation?: string;
  departureTimeOverride?: string;
  teamName: string;
  templates?: Partial<PollTemplates>;
}

export interface PollDraft {
  kind: PollDraftKind;
  text: string;
}

export const defaultPollTemplates: PollTemplates = {
  awayGame:
    'Hoi allemaal!\n\nOp {{date}} speelt {{team}} om {{time}} uit tegen {{opponent}} in {{location}}.\n\nKun je erbij zijn? Reageer met ✅ of ❌. Kun je rijden? Reageer dan ook met 🚗.\nVertrek om {{departureTime}}{{departureLocation}}.',
  homeGame:
    'Hoi allemaal!\n\nOp {{date}} speelt {{team}} om {{time}} thuis tegen {{opponent}} bij {{location}}.\n\nKun je erbij zijn? Reageer met ✅ of ❌.',
};

const formatDate = (date: string): string =>
  new Intl.DateTimeFormat('nl-NL', {
    day: 'numeric',
    month: 'long',
    timeZone: 'UTC',
    weekday: 'long',
  }).format(new Date(`${date}T00:00:00.000Z`));

const renderTemplate = (template: string, values: Record<string, string>): string =>
  template.replace(/\{\{(\w+)\}\}/g, (_, key: string) => values[key] ?? `{{${key}}}`);

export const generatePollDraft = (
  event: ProgramGameEvent,
  options: PollDraftOptions,
): PollDraft => {
  const kind = event.homeTeamName === options.teamName ? 'home' : 'away';
  const departureLocation = options.departureLocation?.trim()
    ? ` vanaf ${options.departureLocation.trim()}`
    : '';
  const template =
    kind === 'home'
      ? (options.templates?.homeGame ?? defaultPollTemplates.homeGame)
      : (options.templates?.awayGame ?? defaultPollTemplates.awayGame);

  return {
    kind,
    text: renderTemplate(template, {
      date: formatDate(event.date),
      departureLocation,
      departureTime: options.departureTimeOverride?.trim() || event.suggestedDepartureTime,
      location: event.locationName,
      opponent: kind === 'home' ? event.awayTeamName : event.homeTeamName,
      team: options.teamName,
      time: event.startTime,
    }),
  };
};
