import type { RequestEvent } from '@sveltejs/kit';
import { describe, expect, it, vi } from 'vitest';

import { createSessionToken } from '$lib/server/authentication/session';

const mocks = vi.hoisted(() => ({
  env: {
    ORIGIN: 'https://team.example.test',
    OIDC_CLIENT_ID: 'team-coordinator',
    OIDC_CLIENT_SECRET: 'client-secret',
    OIDC_ISSUER_URL: 'https://pocket-id.example.test',
    SESSION_SECRET: 'a-secure-session-secret-with-at-least-thirty-two-characters',
    SESSION_TTL_SECONDS: '28800',
    DEV_AUTH_BYPASS: undefined as string | undefined,
  },
}));

vi.mock('$env/dynamic/private', () => ({ env: mocks.env }));
vi.mock('$app/environment', () => ({ dev: true }));

import { handle } from './hooks.server';

const requestEvent = (pathname: string, hasCoordinatorSession = false, sessionToken?: string) =>
  ({
    cookies: { get: () => sessionToken },
    url: new URL(pathname, 'http://localhost'),
    locals: hasCoordinatorSession ? { coordinatorSession: { subject: 'coordinator-1' } } : {},
  }) as unknown as RequestEvent;

describe('default-deny API access', () => {
  it('returns 401 without calling the route handler for an anonymous unlisted API request', async () => {
    const resolve = vi.fn(async () => new Response('route handler reached'));

    const response = await handle({
      event: requestEvent('/api/players'),
      resolve,
    });

    expect(response.status).toBe(401);
    expect(response.headers.get('content-type')).toContain('application/json');
    await expect(response.json()).resolves.toEqual({ error: 'coordinator_session_required' });
    expect(resolve).not.toHaveBeenCalled();
  });

  it('calls the route handler for an authenticated coordinator API request', async () => {
    const resolve = vi.fn(async () => new Response('route handler reached'));

    const response = await handle({
      event: requestEvent('/api/players', true),
      resolve,
    });

    expect(response.status).toBe(200);
    expect(resolve).toHaveBeenCalledOnce();
  });

  it('calls the route handler for the unauthenticated liveness endpoint', async () => {
    const resolve = vi.fn(async () => new Response('route handler reached'));

    const response = await handle({
      event: requestEvent('/api/health/liveness'),
      resolve,
    });

    expect(response.status).toBe(200);
    expect(resolve).toHaveBeenCalledOnce();
  });

  it('bypasses authentication for local development when explicitly enabled', async () => {
    mocks.env.DEV_AUTH_BYPASS = 'true';
    const resolve = vi.fn(async () => new Response('route handler reached'));
    const event = requestEvent('/api/players');

    const response = await handle({ event, resolve });

    expect(response.status).toBe(200);
    expect(event.locals.coordinatorSession).toEqual({ subject: 'local-development' });
    expect(resolve).toHaveBeenCalledOnce();
    delete mocks.env.DEV_AUTH_BYPASS;
  });

  it('loads a valid signed coordinator session from the session cookie', async () => {
    const resolve = vi.fn(async () => new Response('route handler reached'));
    const sessionToken = createSessionToken(
      {
        expiresAt: new Date(Date.now() + 60_000),
        subject: 'pocket-id-subject',
      },
      'a-secure-session-secret-with-at-least-thirty-two-characters',
    );
    const event = requestEvent('/api/players', false, sessionToken);

    const response = await handle({ event, resolve });

    expect(response.status).toBe(200);
    expect(event.locals.coordinatorSession).toEqual({ subject: 'pocket-id-subject' });
    expect(resolve).toHaveBeenCalledOnce();
  });
});
