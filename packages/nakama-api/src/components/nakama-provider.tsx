import { createContext, useState, type ReactNode } from 'react';

import { NakamaApi } from '../api';

export const NakamaContext = createContext<NakamaApi | null>(null);

export function NakamaProvider({ children, api }: { children: ReactNode; api?: NakamaApi }) {
  const [client] = useState(() => api ?? new NakamaApi());

  return <NakamaContext.Provider value={client}>{children}</NakamaContext.Provider>;
}
