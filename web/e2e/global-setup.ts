// A fresh database for each run, so a choice saved in an earlier run cannot change this one.
// The test servers connect on first use, so the database can be made after they start.
import { execFileSync } from 'node:child_process';
import { e2eDatabase } from './database';

export default function globalSetup() {
	const env = { ...process.env, DATABASE_URL: e2eDatabase };
	const run = (args: string[]) => execFileSync('sqlx', args, { cwd: '..', env, stdio: 'inherit' });
	run(['database', 'drop', '-y', '--force']);
	run(['database', 'create']);
	run(['migrate', 'run', '--source', 'migrations']);
}
