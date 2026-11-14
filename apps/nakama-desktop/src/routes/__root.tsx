import { Outlet, createRootRoute } from '@tanstack/react-router';

import { AppHeader } from '../shared/components/app-header';

export const Route = createRootRoute({
  component: RootLayout,
});

function RootLayout() {
  return (
    <div className="flex min-h-svh flex-col">
      <AppHeader />
      <div className="relative min-h-0 flex-1">
        <Outlet />
      </div>
    </div>
  );
}
