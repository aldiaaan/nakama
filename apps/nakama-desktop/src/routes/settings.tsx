import { createFileRoute } from '@tanstack/react-router';

import { SettingsPage } from '../features/settings/components/settings-page';

export const Route = createFileRoute('/settings')({
  component: SettingsRoute,
});

function SettingsRoute() {
  return (
    <div className="view-main bg-background no-scrollbar absolute inset-0 overflow-y-auto">
      <SettingsPage />
    </div>
  );
}
