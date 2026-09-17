import { env } from '$env/dynamic/private';
import type { Handle } from '@sveltejs/kit';

import { readAuthenticationConfiguration } from '$lib/server/authentication/config';
import { coordinatorSessionCookieName, readSessionToken } from '$lib/server/authentication/session';

const unauthenticatedApiPaths = new Set(['/api/health/liveness', '/api/health/readiness']);

const isApiRequest = (pathname: string) => pathname === '/api' || pathname.startsWith('/api/');

export const handle: Handle = async ({ event, resolve }) => {
  const { pathname } = event.url;
  const sessionToken = event.cookies.get(coordinatorSessionCookieName);

  if (sessionToken && !event.locals.coordinatorSession) {
    const authentication = readAuthenticationConfiguration(env);
    const session = readSessionToken(sessionToken, authentication.sessionSecret);

    if (session) {
      event.locals.coordinatorSession = { subject: session.subject };
    }
  }

  if (
    isApiRequest(pathname) &&
    !unauthenticatedApiPaths.has(pathname) &&
    !event.locals.coordinatorSession
  ) {
    return Response.json({ error: 'coordinator_session_required' }, { status: 401 });
  }

  return resolve(event);
};
