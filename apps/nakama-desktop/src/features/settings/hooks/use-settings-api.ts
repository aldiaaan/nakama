import { useContext } from 'react';

import { SettingsContext } from '../components/settings-provider';

export function useSettingsApi() {
  const store = useContext(SettingsContext);
  if (!store) {
    throw new Error('Settings context not found');
  }
  return store;
}
