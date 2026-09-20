import { json } from '@sveltejs/kit';

import { updateTaskStatus } from '$lib/application/tasks/generate-tasks';
import type { TaskStatus } from '$lib/application/tasks/task-repository';
import { currentTaskRepository } from '$lib/server/composition-root';

const statuses = new Set<TaskStatus>(['completed', 'open']);

export const POST = async ({ params, request }) => {
  try {
    const payload: unknown = await request.json();
    const status =
      typeof payload === 'object' && payload !== null
        ? (payload as Record<string, unknown>).status
        : undefined;

    if (typeof status !== 'string' || !statuses.has(status as TaskStatus)) {
      return json({ error: 'invalid_task_status' }, { status: 400 });
    }

    return json(
      await updateTaskStatus(currentTaskRepository(), params.taskId, status as TaskStatus),
    );
  } catch (error) {
    if (error instanceof Error && error.message.includes('does not exist')) {
      return json({ error: 'task_not_found' }, { status: 400 });
    }

    throw error;
  }
};
