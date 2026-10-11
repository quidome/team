// The browser tests use their own database `team_e2e` on the server in DATABASE_URL.
const base = new URL(process.env.DATABASE_URL ?? 'postgresql://team:team@localhost:5432/team_v2');
base.pathname = '/team_e2e';

export const e2eDatabase = base.toString();
