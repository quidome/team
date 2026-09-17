export interface AuthorizationTransaction {
  codeVerifier: string;
  expiresAt: Date;
  nonce: string;
  state: string;
}

interface SerializedAuthorizationTransaction {
  codeVerifier: string;
  expiresAt: number;
  nonce: string;
  state: string;
}

export const authorizationTransactionCookieName = 'team_oidc_transaction';

export const serializeAuthorizationTransaction = (transaction: AuthorizationTransaction) =>
  Buffer.from(
    JSON.stringify({
      codeVerifier: transaction.codeVerifier,
      expiresAt: Math.floor(transaction.expiresAt.getTime() / 1000),
      nonce: transaction.nonce,
      state: transaction.state,
    } satisfies SerializedAuthorizationTransaction),
  ).toString('base64url');

export const readAuthorizationTransaction = (
  value: string,
  now: Date = new Date(),
): AuthorizationTransaction | undefined => {
  try {
    const transaction = JSON.parse(
      Buffer.from(value, 'base64url').toString('utf8'),
    ) as Partial<SerializedAuthorizationTransaction>;
    const { codeVerifier, expiresAt: expiresAtSeconds, nonce, state } = transaction;

    if (
      typeof codeVerifier !== 'string' ||
      !codeVerifier ||
      typeof expiresAtSeconds !== 'number' ||
      !Number.isSafeInteger(expiresAtSeconds) ||
      typeof nonce !== 'string' ||
      !nonce ||
      typeof state !== 'string' ||
      !state
    ) {
      return undefined;
    }

    const expiresAt = new Date(expiresAtSeconds * 1000);

    if (expiresAt <= now) {
      return undefined;
    }

    return { codeVerifier, expiresAt, nonce, state };
  } catch {
    return undefined;
  }
};
