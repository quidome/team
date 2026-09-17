import * as oidc from 'openid-client';

import type { AuthenticationConfiguration } from './config';

export interface OidcAuthorizationTransaction {
  codeVerifier: string;
  nonce: string;
  state: string;
}

export interface OidcIdentity {
  subject: string;
}

export interface OidcClient {
  completeAuthorization(
    callbackUrl: URL,
    transaction: OidcAuthorizationTransaction,
  ): Promise<OidcIdentity>;
  startAuthorization(): Promise<{
    transaction: OidcAuthorizationTransaction;
    url: URL;
  }>;
}

export class OidcIdentityError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'OidcIdentityError';
  }
}

export const createOidcClient = (authentication: AuthenticationConfiguration): OidcClient => {
  let discoveredConfiguration: Promise<oidc.Configuration> | undefined;

  const providerConfiguration = () => {
    discoveredConfiguration ??= oidc.discovery(
      new URL(authentication.issuerUrl),
      authentication.clientId,
      authentication.clientSecret,
    );

    return discoveredConfiguration;
  };

  return {
    async completeAuthorization(callbackUrl, transaction) {
      const tokens = await oidc.authorizationCodeGrant(await providerConfiguration(), callbackUrl, {
        expectedNonce: transaction.nonce,
        expectedState: transaction.state,
        pkceCodeVerifier: transaction.codeVerifier,
      });
      const subject = tokens.claims()?.sub;

      if (typeof subject !== 'string' || !subject) {
        throw new OidcIdentityError('OpenID Connect response did not include a subject');
      }

      return { subject };
    },

    async startAuthorization() {
      const codeVerifier = oidc.randomPKCECodeVerifier();
      const [codeChallenge, configuration] = await Promise.all([
        oidc.calculatePKCECodeChallenge(codeVerifier),
        providerConfiguration(),
      ]);
      const nonce = oidc.randomNonce();
      const state = oidc.randomState();

      return {
        transaction: { codeVerifier, nonce, state },
        url: oidc.buildAuthorizationUrl(configuration, {
          code_challenge: codeChallenge,
          code_challenge_method: 'S256',
          nonce,
          redirect_uri: authentication.redirectUrl,
          scope: 'openid',
          state,
        }),
      };
    },
  };
};
