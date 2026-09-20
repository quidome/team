import { access } from 'node:fs/promises';
import { execFile } from 'node:child_process';
import { promisify } from 'node:util';

const execFileAsync = promisify(execFile);
const targetDatabase = 'team_backup_verify';

const run = async (command, args) => {
  await execFileAsync(command, args, { stdio: 'inherit' });
};

const main = async () => {
  const backupPath = process.argv[2];

  if (!backupPath) {
    throw new Error('Usage: node scripts/verify-postgres-backup.mjs <backup-path>');
  }

  await access(backupPath);

  const databaseUrlValue = process.env.DATABASE_URL;

  if (!databaseUrlValue) {
    throw new Error('DATABASE_URL must be set');
  }

  const databaseUrl = new URL(databaseUrlValue);

  if (!['localhost', '127.0.0.1', '[::1]'].includes(databaseUrl.hostname)) {
    throw new Error('Backup restore verification is restricted to a local PostgreSQL host');
  }

  const adminUrl = new URL(databaseUrl);
  adminUrl.pathname = '/postgres';
  const quotedTarget = `"${targetDatabase}"`;
  const containerBackupPath = `/tmp/${targetDatabase}.dump`;

  await run('psql', [
    adminUrl.toString(),
    '-v',
    'ON_ERROR_STOP=1',
    '-c',
    `DROP DATABASE IF EXISTS ${quotedTarget} WITH (FORCE);`,
    '-c',
    `CREATE DATABASE ${quotedTarget};`,
  ]);

  try {
    await run('docker', ['compose', 'cp', backupPath, `postgres:${containerBackupPath}`]);
    await run('docker', [
      'compose',
      'exec',
      '-T',
      'postgres',
      'pg_restore',
      '--exit-on-error',
      '--no-owner',
      '--username',
      'team',
      '--dbname',
      targetDatabase,
      containerBackupPath,
    ]);
  } finally {
    await run('docker', ['compose', 'exec', '-T', 'postgres', 'rm', '-f', containerBackupPath]);
    await run('psql', [
      adminUrl.toString(),
      '-v',
      'ON_ERROR_STOP=1',
      '-c',
      `DROP DATABASE IF EXISTS ${quotedTarget} WITH (FORCE);`,
    ]);
  }
};

try {
  await main();
} catch (error) {
  console.error(error instanceof Error ? error.message : 'Backup restore verification failed');
  process.exitCode = 1;
}
