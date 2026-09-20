import type { HistoryDenominator } from '$lib/application/history/read-history';
import { loadHistory } from '$lib/server/load-history';

const readDenominator = (value: string | null): HistoryDenominator =>
  value === 'scheduled' ? 'scheduled' : 'recorded';

export const load = async ({ url }) =>
  loadHistory(readDenominator(url.searchParams.get('denominator')));
