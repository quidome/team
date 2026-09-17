import { env } from '$env/dynamic/private';
import { error, redirect } from '@sveltejs/kit';

import { readAuthenticationConfiguration } from '$lib/server/authentication/config';
import { createOidcClient, type OidcIdentity } from '$lib/server/authentication/oidc';
import {
  coordinatorSessionCookieName,
  createSessionToken,
} from '$lib/server/authentication/session';
import {
  authorizationTransactionCookieName,
  readAuthorizationTransaction,
} from '$lib/server/authentication/transaction';

export const GET = async ({ cookies, url }) => {
  const transactionValue = cookies.get(authorizationTransactionCookieName);
  cookies.delete(authorizationTransactionCookieName, { path: '/auth' });

  if (!transactionValue) {
    error(400, 'Missing OpenID Connect authorization transaction');
  }

  const transaction = readAuthorizationTransaction(transactionValue);

  if (!transaction) {
    error(400, 'Invalid or expired OpenID Connect authorization transaction');
  }

  const authentication = readAuthenticationConfiguration(env);
  let identity: OidcIdentity;

  try {
    identity = await createOidcClient(authentication).completeAuthorization(url, transaction);
  } catch {
    error(401, 'OpenID Connect authentication failed');
  }

  const expiresAt = new Date(Date.now() + authentication.sessionTtlSeconds * 1000);
  const token = createSessionToken(
    { expiresAt, subject: identity.subject },
    authentication.sessionSecret,
  );

  cookies.set(coordinatorSessionCookieName, token, {
    httpOnly: true,
    maxAge: authentication.sessionTtlSeconds,
    path: '/',
    sameSite: 'lax',
    secure: authentication.applicationOrigin.startsWith('https://'),
  });

  redirect(303, '/');
};
