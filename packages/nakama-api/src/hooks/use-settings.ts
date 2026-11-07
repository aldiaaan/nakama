import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';

import type { SettingsTheme, SettingsView } from '../resources/settings';
import { formatApiError } from '../utils/format';
import { useNakama } from './use-nakama';

export const settingsQueryKey = ['settings'] as const;

export function useLoadSettings() {
  const api = useNakama();
  const query = useQuery({
    queryKey: settingsQueryKey,
    queryFn: () => api.settings.load(),
    retry: false,
  });

  return {
    ...query,
    errorMessage: formatApiError(query.error),
    statusLabel: query.isPending ? 'Reading config.ini…' : null,
  };
}

export function useSaveSettings() {
  const api = useNakama();
  const queryClient = useQueryClient();
  const mutation = useMutation({
    mutationFn: (theme: SettingsTheme) => api.settings.save(theme),
    onSuccess: (settings: SettingsView) => {
      queryClient.setQueryData(settingsQueryKey, settings);
    },
  });

  return {
    ...mutation,
    errorMessage: formatApiError(mutation.error),
    statusLabel: mutation.isPending ? 'Saving config.ini…' : null,
  };
}
