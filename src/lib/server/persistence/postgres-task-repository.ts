import { asc, eq } from 'drizzle-orm';

import type { Task, TaskRepository, TaskStatus } from '../../application/tasks/task-repository';
import { createDatabase } from './database';
import { tasks } from './schema';

type Database = ReturnType<typeof createDatabase>;

const toTask = (task: {
  description: string | null;
  dueDate: string | null;
  id: string;
  occurrenceId: string | null;
  source: Task['source'];
  status: TaskStatus;
  title: string;
}): Task => ({
  ...(task.description === null ? {} : { description: task.description }),
  ...(task.dueDate === null ? {} : { dueDate: task.dueDate }),
  id: task.id,
  ...(task.occurrenceId === null ? {} : { occurrenceId: task.occurrenceId }),
  source: task.source,
  status: task.status,
  title: task.title,
});

export const createPostgresTaskRepository = (database: Database): TaskRepository => ({
  async findAll() {
    const rows = await database
      .select({
        description: tasks.description,
        dueDate: tasks.dueDate,
        id: tasks.id,
        occurrenceId: tasks.occurrenceId,
        source: tasks.source,
        status: tasks.status,
        title: tasks.title,
      })
      .from(tasks)
      .orderBy(asc(tasks.status), asc(tasks.dueDate), asc(tasks.createdAt));

    return rows.map(toTask);
  },

  async save(task) {
    const [storedTask] = await database
      .insert(tasks)
      .values({
        description: task.description,
        dueDate: task.dueDate,
        occurrenceId: task.occurrenceId,
        source: task.source,
        status: task.status,
        title: task.title,
      })
      .returning({
        description: tasks.description,
        dueDate: tasks.dueDate,
        id: tasks.id,
        occurrenceId: tasks.occurrenceId,
        source: tasks.source,
        status: tasks.status,
        title: tasks.title,
      });

    if (!storedTask) {
      throw new Error('PostgreSQL did not return the stored task');
    }

    return toTask(storedTask);
  },

  async updateStatus(id, status) {
    const [updatedTask] = await database
      .update(tasks)
      .set({ completedAt: status === 'completed' ? new Date() : null, status })
      .where(eq(tasks.id, id))
      .returning({
        description: tasks.description,
        dueDate: tasks.dueDate,
        id: tasks.id,
        occurrenceId: tasks.occurrenceId,
        source: tasks.source,
        status: tasks.status,
        title: tasks.title,
      });

    if (!updatedTask) {
      throw new Error(`Task ${id} does not exist`);
    }

    return toTask(updatedTask);
  },
});
