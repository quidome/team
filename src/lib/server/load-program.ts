import { env } from '$env/dynamic/private';

import { readProgram } from '$lib/application/program/read-program';
import type { ProgramEvent } from '$lib/application/program/read-program';
import {
  currentGameRepository,
  currentTrainingSeriesRepository,
} from '$lib/server/composition-root';

export const loadProgram = async (): Promise<{ events: ProgramEvent[] }> => {
  if (!env.DATABASE_URL?.trim()) {
    return { events: [] };
  }

  return {
    events: await readProgram(currentGameRepository(), currentTrainingSeriesRepository()),
  };
};
