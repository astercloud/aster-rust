import { useCallback, useEffect, useState } from 'react';
import { getRuntimeSnapshot, MessageEvent, Session } from '../api';
import {
  applyModelChangeEvent,
  applyRuntimeSnapshot,
  applyTurnContextEvent,
  createRuntimeInfoFromSession,
  SessionExecutionRuntimeInfo,
} from '../utils/sessionExecutionRuntime';

interface UseSessionExecutionRuntimeResult {
  currentModelInfo: SessionExecutionRuntimeInfo | null;
  syncSession: (session?: Session | null) => void;
  handleStreamEvent: (event: MessageEvent) => void;
}

export function useSessionExecutionRuntime(sessionId: string): UseSessionExecutionRuntimeResult {
  const [currentModelInfo, setCurrentModelInfo] = useState<SessionExecutionRuntimeInfo | null>(
    null
  );

  useEffect(() => {
    setCurrentModelInfo(null);
  }, [sessionId]);

  useEffect(() => {
    if (!sessionId) {
      return;
    }

    let cancelled = false;

    (async () => {
      try {
        const response = await getRuntimeSnapshot({
          path: {
            session_id: sessionId,
          },
          throwOnError: true,
        });

        if (cancelled) {
          return;
        }

        setCurrentModelInfo((current) => applyRuntimeSnapshot(current, response.data));
      } catch (error) {
        console.error('Failed to load runtime snapshot:', error);
      }
    })();

    return () => {
      cancelled = true;
    };
  }, [sessionId]);

  const syncSession = useCallback((session?: Session | null) => {
    const sessionInfo = createRuntimeInfoFromSession(session);
    setCurrentModelInfo((current) => {
      if (!sessionInfo) {
        return current;
      }

      return {
        model: sessionInfo.model ?? current?.model ?? null,
        provider: sessionInfo.provider ?? current?.provider ?? null,
        mode: current?.mode ?? null,
        outputSchemaRuntime: current?.outputSchemaRuntime ?? null,
        source: current?.source ?? sessionInfo.source,
      };
    });
  }, []);

  const handleStreamEvent = useCallback((event: MessageEvent) => {
    switch (event.type) {
      case 'TurnContext':
        setCurrentModelInfo((current) => applyTurnContextEvent(current, event));
        break;
      case 'ModelChange':
        setCurrentModelInfo((current) => applyModelChangeEvent(current, event));
        break;
      default:
        break;
    }
  }, []);

  return {
    currentModelInfo,
    syncSession,
    handleStreamEvent,
  };
}
