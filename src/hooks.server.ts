import { env } from '$env/dynamic/private';
import { dev } from '$app/environment';
import type { Handle } from '@sveltejs/kit';

import { readAuthenticationConfiguration } from '$lib/server/authentication/config';
import { coordinatorSessionCookieName, readSessionToken } from '$lib/server/authentication/session';
import { registerDatabaseShutdown } from '$lib/server/composition-root';

registerDatabaseShutdown();

const unauthenticatedApiPaths = new Set(['/api/health/liveness', '/api/health/readiness']);
const unauthenticatedPagePaths = new Set(['/auth/login', '/auth/callback', '/auth/logout']);

const isApiRequest = (pathname: string) => pathname === '/api' || pathname.startsWith('/api/');

export const handle: Handle = async ({ event, resolve }) => {
  const { pathname } = event.url;
  const sessionToken = event.cookies.get(coordinatorSessionCookieName);
  const developmentAuthBypass = dev && env.DEV_AUTH_BYPASS?.trim().toLowerCase() === 'true';

  if (developmentAuthBypass && !event.locals.coordinatorSession) {
    event.locals.coordinatorSession = { subject: 'local-development' };
  }

  if (sessionToken && !event.locals.coordinatorSession) {
    const authentication = readAuthenticationConfiguration(env);
    const session = readSessionToken(sessionToken, authentication.sessionSecret);

    if (session) {
      event.locals.coordinatorSession = { subject: session.subject };
    }
  }

  if (event.locals.coordinatorSession) {
    return resolve(event);
  }

  if (isApiRequest(pathname)) {
    if (unauthenticatedApiPaths.has(pathname)) {
      return resolve(event);
    }

    return Response.json({ error: 'coordinator_session_required' }, { status: 401 });
  }

  if (unauthenticatedPagePaths.has(pathname)) {
    return resolve(event);
  }

  return new Response(null, { headers: { location: '/auth/login' }, status: 303 });
};
