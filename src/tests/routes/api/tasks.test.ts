import { beforeEach, describe, expect, it, vi } from 'vitest';

const mocks = vi.hoisted(() => ({
  audit: {
    record: vi.fn(),
  },
  tasks: {
    findAll: vi.fn(),
    save: vi.fn(),
    updateStatus: vi.fn(),
  },
}));

vi.mock('$lib/server/composition-root', () => ({
  currentAuditRepository: () => mocks.audit,
  currentTaskRepository: () => mocks.tasks,
}));

import { GET, POST } from '../../../routes/api/tasks/+server';

describe('/api/tasks', () => {
  beforeEach(() => {
    vi.resetAllMocks();
    mocks.tasks.findAll.mockResolvedValue([]);
    mocks.tasks.save.mockImplementation(async (task) => ({ ...task, id: 'task-1' }));
  });

  it('creates a manual reminder', async () => {
    const response = await POST({
      request: new Request('http://localhost/api/tasks', {
        body: JSON.stringify({ dueDate: '2026-08-10', title: 'Confirm rides' }),
        headers: { 'content-type': 'application/json' },
        method: 'POST',
      }),
    } as never);

    expect(response.status).toBe(200);
    expect(mocks.tasks.save).toHaveBeenCalledWith({
      dueDate: '2026-08-10',
      source: 'manual',
      status: 'open',
      title: 'Confirm rides',
    });
    expect(mocks.audit.record).toHaveBeenCalledWith(
      expect.objectContaining({ action: 'task_created', entityId: 'task-1' }),
    );
  });

  it('rejects an invalid due date', async () => {
    const response = await POST({
      request: new Request('http://localhost/api/tasks', {
        body: JSON.stringify({ dueDate: 'tomorrow', title: 'Confirm rides' }),
        headers: { 'content-type': 'application/json' },
        method: 'POST',
      }),
    } as never);

    expect(response.status).toBe(400);
    await expect(response.json()).resolves.toEqual({ error: 'invalid_task' });
    expect(mocks.tasks.save).not.toHaveBeenCalled();
  });

  it('lists manual reminders', async () => {
    mocks.tasks.findAll.mockResolvedValue([
      { id: 'task-1', status: 'open', title: 'Confirm rides' },
    ]);

    const response = await GET();

    expect(response.status).toBe(200);
    await expect(response.json()).resolves.toEqual([
      { id: 'task-1', status: 'open', title: 'Confirm rides' },
    ]);
  });
});
