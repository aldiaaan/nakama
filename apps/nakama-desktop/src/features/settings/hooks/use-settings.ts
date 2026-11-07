import { shallow } from 'zustand/shallow';
import { useStoreWithEqualityFn } from 'zustand/traditional';

import type { SettingsStore } from '../components/settings-provider';
import { useSettingsApi } from './use-settings-api';

export function useSettings(): SettingsStore;
export function useSettings<T>(selector: (state: SettingsStore) => T): T;
export function useSettings<T>(selector?: (state: SettingsStore) => T) {
  const store = useSettingsApi();
  return useStoreWithEqualityFn(store, selector ?? ((state) => state as T), shallow);
}
