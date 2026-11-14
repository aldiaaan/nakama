import { useLoadSettings, type SettingsTheme } from '@nakama/api';
import { isTauri } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { createContext, useEffect, useRef, useState } from 'react';
import { createStore, type StoreApi } from 'zustand/vanilla';

export type SettingsProviderProps = {
  children: React.ReactNode;
};

type SettingsState = {
  theme: SettingsTheme;
  path: string;
};

export type SettingsStore = SettingsState;

export const SettingsContext = createContext<StoreApi<SettingsStore> | null>(null);

export function SettingsProvider({ children }: SettingsProviderProps) {
  const loaded = useLoadSettings();
  const [store] = useState(() =>
    createStore<SettingsStore>()(() => ({
      theme: 'light',
      path: '',
    }))
  );
  const hydrated = useRef(false);

  useEffect(() => {
    if (!loaded.data || hydrated.current) {
      return;
    }
    hydrated.current = true;
    store.setState({ theme: loaded.data.theme, path: loaded.data.path });
  }, [loaded.data, store]);

  useEffect(() => {
    const apply = () => {
      const theme = store.getState().theme;
      document.documentElement.classList.toggle('dark', theme === 'dark');
      if (isTauri() && hydrated.current) {
        void getCurrentWindow().setTheme(theme);
      }
    };
    apply();
    return store.subscribe(apply);
  }, [store]);

  return <SettingsContext.Provider value={store}>{children}</SettingsContext.Provider>;
}
