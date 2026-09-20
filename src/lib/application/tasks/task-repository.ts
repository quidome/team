export type TaskSource = 'generated' | 'manual';
export type TaskStatus = 'completed' | 'open';

export interface Task {
  description?: string;
  dueDate?: string;
  id: string;
  occurrenceId?: string;
  source: TaskSource;
  status: TaskStatus;
  title: string;
}

export interface TaskRepository {
  findAll(): Promise<Task[]>;
  save(task: Omit<Task, 'id'>): Promise<Task>;
  updateStatus(id: string, status: TaskStatus): Promise<Task>;
}
