import { createContext, useContext } from 'react';
import type { SessionExecutionRuntimeInfo } from '../utils/sessionExecutionRuntime';

export const SessionExecutionContext = createContext<SessionExecutionRuntimeInfo | null>(null);

export function useCurrentModelInfo(): SessionExecutionRuntimeInfo | null {
  return useContext(SessionExecutionContext);
}
