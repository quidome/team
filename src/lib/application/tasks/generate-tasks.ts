import type { ProgramEvent } from '../program/read-program';
import type { Task, TaskRepository } from './task-repository';

export interface GeneratedTaskOptions {
  teamName: string;
  today: string;
}

const eventTitle = (event: ProgramEvent): string => {
  if (event.type === 'training') {
    return 'Team training';
  }

  return `${event.homeTeamName} vs ${event.awayTeamName}`;
};

export const generateTasks = (events: ProgramEvent[], options: GeneratedTaskOptions): Task[] =>
  events
    .filter(
      (event) =>
        event.date >= options.today && (event.type !== 'game' || event.status === 'scheduled'),
    )
    .slice(0, 12)
    .map((event) => ({
      description:
        event.type === 'game' && event.homeTeamName === options.teamName
          ? 'Prepare the home-game attendance poll draft.'
          : event.type === 'game'
            ? 'Prepare the away-game attendance and transport poll draft.'
            : 'Record attendance after the training session.',
      dueDate: event.date,
      id: `generated:${event.id}`,
      occurrenceId: event.id,
      source: 'generated' as const,
      status: 'open' as const,
      title: `Coordinate ${eventTitle(event)}`,
    }));

export const readTasks = async (
  tasks: TaskRepository,
  events: ProgramEvent[],
  options: GeneratedTaskOptions,
): Promise<Task[]> => {
  const manualTasks = await tasks.findAll();
  const generatedTasks = generateTasks(events, options);

  return [...manualTasks, ...generatedTasks].sort((left, right) =>
    `${left.dueDate ?? '9999-12-31'}:${left.title}`.localeCompare(
      `${right.dueDate ?? '9999-12-31'}:${right.title}`,
    ),
  );
};

export const createManualTask = (
  tasks: TaskRepository,
  task: Omit<Task, 'id' | 'source' | 'status'>,
): Promise<Task> => tasks.save({ ...task, source: 'manual', status: 'open' });

export const updateTaskStatus = (
  tasks: TaskRepository,
  id: string,
  status: 'completed' | 'open',
): Promise<Task> => tasks.updateStatus(id, status);
