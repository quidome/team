import { describe, expect, it, vi } from 'vitest';

import { coordinatorSessionCookieName } from '$lib/server/authentication/session';

import { POST } from '../../../routes/auth/logout/+server';

describe('POST /auth/logout', () => {
  it('deletes the coordinator session before redirecting home', async () => {
    const cookies = { delete: vi.fn() };

    await expect(POST({ cookies } as never)).rejects.toMatchObject({
      location: '/',
      status: 303,
    });
    expect(cookies.delete).toHaveBeenCalledWith(coordinatorSessionCookieName, { path: '/' });
  });
});
