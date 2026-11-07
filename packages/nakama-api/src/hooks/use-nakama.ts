import { useContext } from 'react';

import type { NakamaApi } from '../api';
import { NakamaContext } from '../components/nakama-provider';

export function useNakama(): NakamaApi {
  const api = useContext(NakamaContext);
  if (!api) {
    throw new Error('useNakama must be used within NakamaProvider');
  }
  return api;
}
