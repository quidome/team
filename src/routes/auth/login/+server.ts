import { env } from '$env/dynamic/private';
import { redirect } from '@sveltejs/kit';

import { readAuthenticationConfiguration } from '$lib/server/authentication/config';
import { createOidcClient } from '$lib/server/authentication/oidc';
import {
  authorizationTransactionCookieName,
  serializeAuthorizationTransaction,
} from '$lib/server/authentication/transaction';

const authorizationTransactionTtlSeconds = 5 * 60;

export const GET = async ({ cookies }) => {
  const authentication = readAuthenticationConfiguration(env);
  const authorization = await createOidcClient(authentication).startAuthorization();

  cookies.set(
    authorizationTransactionCookieName,
    serializeAuthorizationTransaction({
      ...authorization.transaction,
      expiresAt: new Date(Date.now() + authorizationTransactionTtlSeconds * 1000),
    }),
    {
      httpOnly: true,
      maxAge: authorizationTransactionTtlSeconds,
      path: '/auth',
      sameSite: 'lax',
      secure: authentication.applicationOrigin.startsWith('https://'),
    },
  );

  redirect(303, authorization.url.toString());
};
