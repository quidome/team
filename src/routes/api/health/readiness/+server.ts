import { json } from '@sveltejs/kit';

export const GET = () =>
  json(
    { status: 'ready' },
    {
      headers: {
        'cache-control': 'no-store',
      },
    },
  );
