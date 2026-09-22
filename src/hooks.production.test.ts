import type { RequestEvent } from '@sveltejs/kit';
import { describe, expect, it, vi } from 'vitest';

const mocks = vi.hoisted(() => ({
  environment: {
    DEV_AUTH_BYPASS: 'true',
  },
}));

vi.mock('$env/dynamic/private', () => ({ env: mocks.environment }));
vi.mock('$app/environment', () => ({ dev: false }));

import { handle } from './hooks.server';

const requestEvent = (pathname = '/api/players') =>
  ({
    cookies: { get: () => undefined },
    locals: {},
    url: new URL(pathname, 'https://team.example.test'),
  }) as unknown as RequestEvent;

describe('production authentication guard', () => {
  it('ignores the development bypass outside Vite development mode', async () => {
    const resolve = vi.fn(async () => new Response('route handler reached'));

    const response = await handle({ event: requestEvent(), resolve });

    expect(response.status).toBe(401);
    await expect(response.json()).resolves.toEqual({ error: 'coordinator_session_required' });
    expect(resolve).not.toHaveBeenCalled();
  });

  it('redirects an anonymous page request to login outside Vite development mode', async () => {
    const resolve = vi.fn(async () => new Response('route handler reached'));

    const response = await handle({ event: requestEvent('/'), resolve });

    expect(response.status).toBe(303);
    expect(response.headers.get('location')).toBe('/auth/login');
    expect(resolve).not.toHaveBeenCalled();
  });
});
