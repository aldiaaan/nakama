import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';

import type { IpcRequestType, ProfilerKind } from '../resources/daemon';
import { formatApiError } from '../utils/format';
import { useNakama } from './use-nakama';

export const daemonOverviewQueryKey = ['daemon-overview'] as const;

const profilerNames: Record<ProfilerKind, string> = {
  cpu: 'CPU',
  heap: 'heap',
};

export function useDaemonOverview() {
  const api = useNakama();
  const query = useQuery({
    queryKey: daemonOverviewQueryKey,
    queryFn: () => api.daemon.overview(),
    retry: false,
    refetchInterval: 1000,
  });

  return {
    ...query,
    errorMessage: formatApiError(query.error),
    statusLabel: query.isPending ? 'Connecting to the daemon…' : null,
  };
}

export function useStartDaemon() {
  const api = useNakama();
  const mutation = useDaemonMutation(() => api.daemon.start());
  return {
    ...mutation,
    errorMessage: formatApiError(mutation.error),
    statusLabel: mutation.isPending ? 'Starting the daemon…' : null,
  };
}

export function useDaemonIpcRequest() {
  const api = useNakama();
  const mutation = useDaemonMutation((type: IpcRequestType) => api.daemon.sendIpc(type));
  return {
    ...mutation,
    errorMessage: formatApiError(mutation.error),
    statusLabel: mutation.isPending ? `Sending ${mutation.variables} over IPC…` : null,
  };
}

export function useStartProfiler() {
  const api = useNakama();
  const mutation = useDaemonMutation((kind: ProfilerKind) => api.daemon.startProfiler(kind));
  return {
    ...mutation,
    errorMessage: formatApiError(mutation.error),
    statusLabel: mutation.isPending ? `Starting the ${profilerNames[mutation.variables]} profiler…` : null,
  };
}

export function useStopProfiler() {
  const api = useNakama();
  const mutation = useDaemonMutation((kind: ProfilerKind) => api.daemon.stopProfiler(kind));
  return {
    ...mutation,
    errorMessage: formatApiError(mutation.error),
    statusLabel: mutation.isPending ? `Writing the ${profilerNames[mutation.variables]} profile…` : null,
  };
}

/** Refreshes the overview once the command settles, so the page shows its effect right away. */
function useDaemonMutation<TVariables, TData>(mutationFn: (variables: TVariables) => Promise<TData>) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn,
    onSettled: () => queryClient.invalidateQueries({ queryKey: daemonOverviewQueryKey }),
  });
}
