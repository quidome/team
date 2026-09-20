import { loadLocations } from '$lib/server/load-settings';
import { loadProgram } from '$lib/server/load-program';

export const load = async () => {
  const [program, locations] = await Promise.all([loadProgram(), loadLocations()]);

  return { ...program, locations };
};
