import { beforeEach, describe, expect, it, vi } from 'vitest';

const mocks = vi.hoisted(() => ({
  audit: {
    record: vi.fn(),
  },
  tasks: {
    updateStatus: vi.fn(),
  },
}));

vi.mock('$lib/server/composition-root', () => ({
  currentAuditRepository: () => mocks.audit,
  currentTaskRepository: () => mocks.tasks,
}));

import { POST } from '../../../routes/api/tasks/[taskId]/status/+server';

describe('POST /api/tasks/:taskId/status', () => {
  beforeEach(() => {
    vi.resetAllMocks();
    mocks.tasks.updateStatus.mockResolvedValue({
      id: 'task-1',
      source: 'manual',
      status: 'completed',
      title: 'Confirm rides',
    });
  });

  it('records a task status audit entry', async () => {
    const response = await POST({
      params: { taskId: 'task-1' },
      request: new Request('http://localhost/api/tasks/task-1/status', {
        body: JSON.stringify({ status: 'completed' }),
        headers: { 'content-type': 'application/json' },
        method: 'POST',
      }),
    } as never);

    expect(response.status).toBe(200);
    expect(mocks.audit.record).toHaveBeenCalledWith(
      expect.objectContaining({
        action: 'task_status_changed',
        entityId: 'task-1',
      }),
    );
  });
});
