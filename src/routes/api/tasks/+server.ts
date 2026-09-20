import { json } from '@sveltejs/kit';

import { createManualTask } from '$lib/application/tasks/generate-tasks';
import type { Task } from '$lib/application/tasks/task-repository';
import { currentTaskRepository } from '$lib/server/composition-root';

const isCalendarDate = (value: string) => {
  if (!/^\d{4}-\d{2}-\d{2}$/.test(value)) {
    return false;
  }

  const date = new Date(`${value}T00:00:00.000Z`);

  return Number.isFinite(date.getTime()) && date.toISOString().slice(0, 10) === value;
};

const readTask = async (
  request: Request,
): Promise<Omit<Task, 'id' | 'source' | 'status'> | undefined> => {
  try {
    const payload: unknown = await request.json();

    if (typeof payload !== 'object' || payload === null) {
      return undefined;
    }

    const { description, dueDate, title } = payload as Record<string, unknown>;

    if (
      typeof title !== 'string' ||
      !title.trim() ||
      (description !== undefined && typeof description !== 'string') ||
      (dueDate !== undefined && (typeof dueDate !== 'string' || !isCalendarDate(dueDate)))
    ) {
      return undefined;
    }

    return {
      ...(description && description.trim() ? { description: description.trim() } : {}),
      ...(dueDate ? { dueDate } : {}),
      title: title.trim(),
    };
  } catch {
    return undefined;
  }
};

export const GET = async () => json(await currentTaskRepository().findAll());

export const POST = async ({ request }) => {
  const task = await readTask(request);

  if (!task) {
    return json({ error: 'invalid_task' }, { status: 400 });
  }

  return json(await createManualTask(currentTaskRepository(), task));
};
