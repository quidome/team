import { redirect } from '@sveltejs/kit';

import { coordinatorSessionCookieName } from '$lib/server/authentication/session';

export const POST = async ({ cookies }) => {
  cookies.delete(coordinatorSessionCookieName, { path: '/' });

  redirect(303, '/');
};
