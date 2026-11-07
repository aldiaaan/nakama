import { Link, Outlet, createRootRoute } from '@tanstack/react-router';

export const Route = createRootRoute({
  component: RootLayout,
});

function RootLayout() {
  return (
    <div className="flex min-h-svh flex-col">
      <header className="border-border bg-background/80 sticky top-0 z-10 border-b backdrop-blur">
        <nav className="mx-auto flex w-full max-w-5xl items-center gap-1 px-8 py-3">
          <span className="mr-3 text-sm font-semibold tracking-tight">Nakama</span>
          <NavLink to="/">Home</NavLink>
          <NavLink to="/settings">Settings</NavLink>
        </nav>
      </header>
      <div className="relative min-h-0 flex-1">
        <Outlet />
      </div>
    </div>
  );
}

function NavLink({ to, children }: { to: '/' | '/settings'; children: string }) {
  return (
    <Link
      to={to}
      activeOptions={{ exact: true }}
      className="rounded-full px-3 py-1.5 text-sm transition-colors"
      inactiveProps={{
        className: 'text-muted-foreground hover:bg-muted hover:text-foreground',
      }}
      activeProps={{
        className: 'bg-secondary text-secondary-foreground',
      }}
    >
      {children}
    </Link>
  );
}
