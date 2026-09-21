export const load = async ({ locals }) => ({
  isAuthenticated: Boolean(locals.coordinatorSession),
});
