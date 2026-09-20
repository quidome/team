import { json } from '@sveltejs/kit';

import { updateTaskStatus } from '$lib/application/tasks/generate-tasks';
import type { TaskStatus } from '$lib/application/tasks/task-repository';
import { currentAuditRepository, currentTaskRepository } from '$lib/server/composition-root';

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

    const updatedTask = await updateTaskStatus(
      currentTaskRepository(),
      params.taskId,
      status as TaskStatus,
    );

    await currentAuditRepository().record({
      action: 'task_status_changed',
      entityId: updatedTask.id,
      entityType: 'task',
      metadata: { status: updatedTask.status, title: updatedTask.title },
    });

    return json(updatedTask);
  } catch (error) {
    if (error instanceof Error && error.message.includes('does not exist')) {
      return json({ error: 'task_not_found' }, { status: 400 });
    }

    throw error;
  }
};
