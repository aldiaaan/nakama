import { Link } from '@tanstack/react-router';
import { isTauri } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { useEffect, useState } from 'react';

export function AppHeader() {
  return (
    <header
      // Bare drag regions only drag when the click lands on that element. `deep` covers the bar.
      data-tauri-drag-region="deep"
      className="border-border bg-background/80 sticky top-0 z-10 flex h-12 shrink-0 items-center border-b backdrop-blur select-none"
    >
      <nav className="mx-auto flex h-full w-full max-w-5xl items-center gap-1 pr-40 pl-8">
        <span className="mr-3 text-sm font-semibold tracking-tight">Nakama</span>
        <NavLink to="/">Home</NavLink>
        <NavLink to="/daemon">Daemon</NavLink>
        <NavLink to="/settings">Settings</NavLink>
      </nav>
      <WindowControls />
    </header>
  );
}

function NavLink({ to, children }: { to: '/' | '/daemon' | '/settings'; children: string }) {
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

function WindowControls() {
  const maximized = useWindowMaximized();

  return (
    <div data-tauri-drag-region="false" className="absolute inset-y-0 right-0 flex">
      <CaptionButton label="Minimize" onClick={minimize}>
        <path d="M2 6h8" />
      </CaptionButton>
      <CaptionButton label={maximized ? 'Restore' : 'Maximize'} onClick={toggleMaximize}>
        {maximized ? (
          <>
            <path d="M4.25 2.25h5.5v5.5" />
            <path d="M2.25 4.25h5.5v5.5h-5.5z" />
          </>
        ) : (
          <rect x="2.25" y="2.25" width="7.5" height="7.5" />
        )}
      </CaptionButton>
      <CaptionButton label="Close" onClick={close} close>
        <path d="M3 3 9 9M9 3 3 9" />
      </CaptionButton>
    </div>
  );
}

type CaptionButtonProps = {
  label: string;
  onClick: () => void;
  close?: boolean;
  children: React.ReactNode;
};

function CaptionButton({ label, onClick, close: isClose, children }: CaptionButtonProps) {
  return (
    <button
      type="button"
      aria-label={label}
      title={label}
      onClick={onClick}
      className={
        isClose
          ? 'text-foreground/80 inline-flex h-full w-12 items-center justify-center transition-colors outline-none hover:bg-[#c42b1c] hover:text-white focus-visible:bg-[#c42b1c] focus-visible:text-white'
          : 'text-foreground/80 hover:bg-foreground/10 hover:text-foreground focus-visible:bg-foreground/10 inline-flex h-full w-12 items-center justify-center transition-colors outline-none'
      }
    >
      <svg viewBox="0 0 12 12" className="size-3 fill-none stroke-current stroke-1" aria-hidden>
        {children}
      </svg>
    </button>
  );
}

function useWindowMaximized() {
  const [maximized, setMaximized] = useState(false);

  useEffect(() => {
    if (!isTauri()) {
      return;
    }

    const appWindow = getCurrentWindow();
    let unlisten: (() => void) | undefined;
    let cancelled = false;

    void appWindow.isMaximized().then((value) => {
      if (!cancelled) {
        setMaximized(value);
      }
    });

    void appWindow
      .onResized(() => {
        void appWindow.isMaximized().then((value) => {
          if (!cancelled) {
            setMaximized(value);
          }
        });
      })
      .then((stop) => {
        if (cancelled) {
          stop();
          return;
        }
        unlisten = stop;
      });

    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, []);

  return maximized;
}

function minimize() {
  if (!isTauri()) {
    return;
  }
  void getCurrentWindow().minimize();
}

function toggleMaximize() {
  if (!isTauri()) {
    return;
  }
  void getCurrentWindow().toggleMaximize();
}

function close() {
  if (!isTauri()) {
    return;
  }
  void getCurrentWindow().close();
}
