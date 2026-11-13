import { useMutation } from '@tanstack/react-query';

import type { NotificationPayload } from '../resources/notification';
import { formatApiError } from '../utils/format';
import { useNakama } from './use-nakama';

export function usePushNotification() {
  const api = useNakama();
  const mutation = useMutation({
    mutationFn: (notification: NotificationPayload) => api.notification.push(notification),
  });

  return {
    ...mutation,
    errorMessage: formatApiError(mutation.error),
    statusLabel: mutation.isPending ? 'Showing the notification…' : null,
  };
}
