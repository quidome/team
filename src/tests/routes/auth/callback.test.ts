import { beforeEach, describe, expect, it, vi } from 'vitest';

import { coordinatorSessionCookieName, readSessionToken } from '$lib/server/authentication/session';
import {
  authorizationTransactionCookieName,
  serializeAuthorizationTransaction,
} from '$lib/server/authentication/transaction';

const mocks = vi.hoisted(() => ({
  completeAuthorization: vi.fn(),
}));

vi.mock('$env/dynamic/private', () => ({
  env: {
    ORIGIN: 'https://team.example.test',
    OIDC_CLIENT_ID: 'team-coordinator',
    OIDC_CLIENT_SECRET: 'client-secret',
    OIDC_ISSUER_URL: 'https://pocket-id.example.test',
    SESSION_SECRET: 'a-secure-session-secret-with-at-least-thirty-two-characters',
    SESSION_TTL_SECONDS: '28800',
  },
}));

vi.mock('$lib/server/authentication/oidc', () => ({
  createOidcClient: () => ({
    completeAuthorization: mocks.completeAuthorization,
  }),
}));

import { GET } from '../../../routes/auth/callback/+server';

const sessionSecret = 'a-secure-session-secret-with-at-least-thirty-two-characters';

describe('GET /auth/callback', () => {
  beforeEach(() => {
    vi.resetAllMocks();
    mocks.completeAuthorization.mockResolvedValue({ subject: 'pocket-id-subject' });
  });

  it('exchanges a valid transaction for a secure coordinator session', async () => {
    const cookies = {
      delete: vi.fn(),
      get: vi.fn().mockReturnValue(
        serializeAuthorizationTransaction({
          codeVerifier: 'pkce-verifier',
          expiresAt: new Date(Date.now() + 60_000),
          nonce: 'oidc-nonce',
          state: 'oidc-state',
        }),
      ),
      set: vi.fn(),
    };

    await expect(
      GET({
        cookies,
        url: new URL('https://team.example.test/auth/callback?code=authorization-code'),
      } as never),
    ).rejects.toMatchObject({ location: '/', status: 303 });
    expect(mocks.completeAuthorization).toHaveBeenCalledWith(
      new URL('https://team.example.test/auth/callback?code=authorization-code'),
      expect.objectContaining({
        codeVerifier: 'pkce-verifier',
        nonce: 'oidc-nonce',
        state: 'oidc-state',
      }),
    );
    expect(cookies.delete).toHaveBeenCalledWith(authorizationTransactionCookieName, {
      path: '/auth',
    });

    const [name, value, options] = cookies.set.mock.calls[0] as [string, string, object];
    expect(name).toBe(coordinatorSessionCookieName);
    expect(readSessionToken(value, sessionSecret)).toMatchObject({
      subject: 'pocket-id-subject',
    });
    expect(options).toMatchObject({
      httpOnly: true,
      maxAge: 28800,
      path: '/',
      sameSite: 'lax',
      secure: true,
    });
  });
});
