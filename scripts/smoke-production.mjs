const baseUrl = new URL(process.argv[2] ?? process.env.SMOKE_BASE_URL ?? 'http://127.0.0.1:4173');

const checks = [
  { body: { status: 'ok' }, path: '/api/health/liveness', status: 200 },
  { body: { status: 'ready' }, path: '/api/health/readiness', status: 200 },
  {
    body: { error: 'coordinator_session_required' },
    path: '/api/players',
    status: 401,
  },
];

for (const check of checks) {
  const response = await fetch(new URL(check.path, baseUrl));
  let body;

  try {
    body = await response.json();
  } catch {
    body = undefined;
  }

  if (response.status !== check.status || JSON.stringify(body) !== JSON.stringify(check.body)) {
    throw new Error(
      `${check.path} expected ${check.status} ${JSON.stringify(check.body)}, received ${response.status} ${JSON.stringify(body)}`,
    );
  }

  console.log(`${check.path}: ${response.status}`);
}
