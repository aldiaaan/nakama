import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';

import type { IpcRequestType } from '../resources/daemon';
import { formatApiError } from '../utils/format';
import { useNakama } from './use-nakama';

export function daemonOverviewQueryKey(name: string) {
  return ['daemon', name, 'overview'] as const;
}

export function useDaemonOverview(name: string) {
  const api = useNakama();
  const query = useQuery({
    queryKey: daemonOverviewQueryKey(name),
    queryFn: () => api.daemon.overview(name),
    retry: false,
    refetchInterval: 1000,
  });

  return {
    ...query,
    errorMessage: formatApiError(query.error),
    statusLabel: query.isPending ? `Connecting to ${name}…` : null,
  };
}

export function useStartDaemon(name: string) {
  const api = useNakama();
  const mutation = useDaemonMutation(name, () => api.daemon.start(name));
  return {
    ...mutation,
    errorMessage: formatApiError(mutation.error),
    statusLabel: mutation.isPending ? `Starting ${name}…` : null,
  };
}

export function useDaemonIpcRequest(name: string) {
  const api = useNakama();
  const mutation = useDaemonMutation(name, (type: IpcRequestType) => api.daemon.sendIpc(name, type));
  return {
    ...mutation,
    errorMessage: formatApiError(mutation.error),
    statusLabel: mutation.isPending ? `Sending ${mutation.variables} over IPC…` : null,
  };
}

/** Refreshes the daemon's overview once the command settles, so the page shows its effect right away. */
function useDaemonMutation<TVariables, TData>(name: string, mutationFn: (variables: TVariables) => Promise<TData>) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn,
    onSettled: () => queryClient.invalidateQueries({ queryKey: daemonOverviewQueryKey(name) }),
  });
}
