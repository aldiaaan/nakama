import { useQuery } from '@tanstack/react-query';

import { formatApiError } from '../utils/format';
import { useNakama } from './use-nakama';

export const deviceInfoQueryKey = ['device-info'] as const;

export function useDeviceInfo() {
  const api = useNakama();
  const query = useQuery({
    queryKey: deviceInfoQueryKey,
    queryFn: () => api.device.info(),
    retry: false,
    refetchInterval: 500,
  });

  return {
    ...query,
    errorMessage: formatApiError(query.error),
  };
}
