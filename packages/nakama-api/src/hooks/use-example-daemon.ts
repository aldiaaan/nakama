import { useQuery } from '@tanstack/react-query';

import { formatApiError } from '../utils/format';
import { useNakama } from './use-nakama';

export const exampleDaemonReportQueryKey = ['example-daemon', 'report'] as const;

export function useExampleDaemonReport() {
  const api = useNakama();
  const query = useQuery({
    queryKey: exampleDaemonReportQueryKey,
    queryFn: () => api.exampleDaemon.report(),
    retry: false,
    refetchInterval: 1000,
  });

  return {
    ...query,
    errorMessage: formatApiError(query.error),
    statusLabel: query.isPending ? 'Loading calculations…' : null,
  };
}
