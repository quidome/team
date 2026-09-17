import { json } from '@sveltejs/kit';

export const GET = () =>
  json(
    { status: 'ok' },
    {
      headers: {
        'cache-control': 'no-store',
      },
    },
  );
