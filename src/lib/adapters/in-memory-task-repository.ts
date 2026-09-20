import type { Task, TaskRepository, TaskStatus } from '../application/tasks/task-repository';

export class InMemoryTaskRepository implements TaskRepository {
  private nextId = 1;
  private readonly tasks = new Map<string, Task>();

  constructor(initialTasks: Task[] = []) {
    for (const task of initialTasks) {
      this.tasks.set(task.id, task);
    }
  }

  async findAll(): Promise<Task[]> {
    return [...this.tasks.values()].map((task) => ({ ...task }));
  }

  async save(task: Omit<Task, 'id'>): Promise<Task> {
    const storedTask = { ...task, id: `task-${this.nextId++}` };
    this.tasks.set(storedTask.id, storedTask);

    return storedTask;
  }

  async updateStatus(id: string, status: TaskStatus): Promise<Task> {
    const task = this.tasks.get(id);

    if (!task) {
      throw new Error(`Task ${id} does not exist`);
    }

    task.status = status;

    return { ...task };
  }
}
