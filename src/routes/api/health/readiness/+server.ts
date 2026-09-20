import { json } from '@sveltejs/kit';

import { checkDatabaseConnection } from '$lib/server/composition-root';

export const GET = async () => {
  const ready = await checkDatabaseConnection();

  return json(
    { status: ready ? 'ready' : 'not_ready' },
    {
      headers: {
        'cache-control': 'no-store',
      },
      status: ready ? 200 : 503,
    },
  );
};
