import { createFileRoute } from '@tanstack/react-router';

import { DeviceInfoPanel } from '../features/device/device-info-panel';

export const Route = createFileRoute('/')({
  component: HomePage,
});

function HomePage() {
  return (
    <div className="view-main bg-background no-scrollbar absolute inset-0 overflow-y-auto">
      <DeviceInfoPanel />
    </div>
  );
}
