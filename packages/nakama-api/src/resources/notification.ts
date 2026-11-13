import type { NakamaApi } from '../api';

export type NotificationPayload = {
  title: string;
  body: string;
};

export type NotificationView = {
  title: string;
  body: string;
  summary: string;
};

/** A notification in the operating system's notification center. */
export class NotificationResource {
  constructor(private readonly api: NakamaApi) {}

  async push(notification: NotificationPayload): Promise<NotificationView> {
    const title = notification.title.trim();
    const body = notification.body.trim();
    await this.api.invoke<null>('push_notification', { title, body });
    return {
      title,
      body,
      summary: `${title} — ${body}`,
    };
  }
}
