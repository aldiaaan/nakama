import { createFileRoute } from '@tanstack/react-router';

import { DaemonPage } from '../features/daemon/components/daemon-page';

export const Route = createFileRoute('/daemon')({
  component: DaemonRoute,
});

function DaemonRoute() {
  return (
    <div className="view-main bg-background no-scrollbar absolute inset-0 overflow-y-auto">
      <DaemonPage />
    </div>
  );
}
