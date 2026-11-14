import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';

import type { ProfilerKind } from '../resources/profilers';
import { formatApiError } from '../utils/format';
import { useNakama } from './use-nakama';

export const profilersQueryKey = ['profilers'] as const;

const profilerNames: Record<ProfilerKind, string> = {
  cpu: 'CPU',
  heap: 'heap',
};

export function useProfilers() {
  const api = useNakama();
  const query = useQuery({
    queryKey: profilersQueryKey,
    queryFn: () => api.profilers.list(),
    retry: false,
    refetchInterval: 1000,
  });

  return {
    ...query,
    errorMessage: formatApiError(query.error),
    statusLabel: query.isPending ? 'Loading profilers…' : null,
  };
}

export function useStartProfiler() {
  const api = useNakama();
  const mutation = useProfilerMutation((kind: ProfilerKind) => api.profilers.start(kind));
  return {
    ...mutation,
    errorMessage: formatApiError(mutation.error),
    statusLabel: mutation.isPending ? `Starting the ${profilerNames[mutation.variables]} profiler…` : null,
  };
}

export function useStopProfiler() {
  const api = useNakama();
  const mutation = useProfilerMutation((kind: ProfilerKind) => api.profilers.stop(kind));
  return {
    ...mutation,
    errorMessage: formatApiError(mutation.error),
    statusLabel: mutation.isPending ? `Writing the ${profilerNames[mutation.variables]} profile…` : null,
  };
}

function useProfilerMutation(mutationFn: (kind: ProfilerKind) => Promise<void>) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn,
    onSettled: () => queryClient.invalidateQueries({ queryKey: profilersQueryKey }),
  });
}
