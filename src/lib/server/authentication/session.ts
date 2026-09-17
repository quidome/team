import { createHmac, timingSafeEqual } from 'node:crypto';

export const coordinatorSessionCookieName = 'team_coordinator_session';

export interface CoordinatorSession {
  expiresAt: Date;
  subject: string;
}

interface SessionPayload {
  expiresAt: number;
  subject: string;
  version: 1;
}

const encodePayload = (session: CoordinatorSession) =>
  Buffer.from(
    JSON.stringify({
      expiresAt: Math.floor(session.expiresAt.getTime() / 1000),
      subject: session.subject,
      version: 1,
    } satisfies SessionPayload),
  ).toString('base64url');

const sign = (payload: string, sessionSecret: string) =>
  createHmac('sha256', sessionSecret).update(payload).digest('base64url');

const validSignature = (actual: string, expected: string) => {
  const actualBytes = Buffer.from(actual);
  const expectedBytes = Buffer.from(expected);

  return actualBytes.length === expectedBytes.length && timingSafeEqual(actualBytes, expectedBytes);
};

export const createSessionToken = (session: CoordinatorSession, sessionSecret: string) => {
  const payload = encodePayload(session);

  return `${payload}.${sign(payload, sessionSecret)}`;
};

export const readSessionToken = (
  token: string,
  sessionSecret: string,
  now: Date = new Date(),
): CoordinatorSession | undefined => {
  const [encodedPayload, signature, ...remainingParts] = token.split('.');

  if (!encodedPayload || !signature || remainingParts.length > 0) {
    return undefined;
  }

  if (!validSignature(signature, sign(encodedPayload, sessionSecret))) {
    return undefined;
  }

  try {
    const payload = JSON.parse(
      Buffer.from(encodedPayload, 'base64url').toString('utf8'),
    ) as Partial<SessionPayload>;
    const expiresAtSeconds = payload.expiresAt;
    const subject = payload.subject;

    if (
      payload.version !== 1 ||
      typeof subject !== 'string' ||
      !subject ||
      typeof expiresAtSeconds !== 'number' ||
      !Number.isSafeInteger(expiresAtSeconds)
    ) {
      return undefined;
    }

    const expiresAt = new Date(expiresAtSeconds * 1000);

    if (expiresAt <= now) {
      return undefined;
    }

    return { expiresAt, subject };
  } catch {
    return undefined;
  }
};
