import type {
  MessageEvent,
  Session,
  SessionRuntimeSnapshot,
  TurnOutputSchemaRuntime,
  TurnRuntime,
} from '../api';

export type SessionExecutionRuntimeSource =
  | 'session'
  | 'runtime_snapshot'
  | 'turn_context'
  | 'model_change';

export interface SessionExecutionRuntimeInfo {
  model: string | null;
  provider: string | null;
  mode: string | null;
  outputSchemaRuntime: TurnOutputSchemaRuntime | null;
  source: SessionExecutionRuntimeSource;
}

type TurnContextEvent = Extract<MessageEvent, { type: 'TurnContext' }>;
type ModelChangeEvent = Extract<MessageEvent, { type: 'ModelChange' }>;

function buildRuntimeInfo(
  current: SessionExecutionRuntimeInfo | null,
  updates: Partial<SessionExecutionRuntimeInfo>,
  source: SessionExecutionRuntimeSource
): SessionExecutionRuntimeInfo | null {
  const model = updates.model ?? current?.model ?? null;
  const provider = updates.provider ?? current?.provider ?? null;
  const mode = updates.mode ?? current?.mode ?? null;
  const outputSchemaRuntime = updates.outputSchemaRuntime ?? current?.outputSchemaRuntime ?? null;

  if (!model && !provider && !outputSchemaRuntime) {
    return null;
  }

  return {
    model,
    provider,
    mode,
    outputSchemaRuntime,
    source,
  };
}

function getTurnTimestamp(turn: TurnRuntime): number {
  return Date.parse(turn.updatedAt || turn.startedAt || turn.createdAt || '') || 0;
}

export function createRuntimeInfoFromSession(
  session?: Session | null
): SessionExecutionRuntimeInfo | null {
  if (!session) {
    return null;
  }

  const model = session.model_config?.model_name ?? null;
  const provider = session.provider_name ?? null;

  if (!model && !provider) {
    return null;
  }

  return {
    model,
    provider,
    mode: null,
    outputSchemaRuntime: null,
    source: 'session',
  };
}

export function selectLatestTurnRuntime(
  snapshot?: SessionRuntimeSnapshot | null
): TurnRuntime | null {
  if (!snapshot?.threads?.length) {
    return null;
  }

  const turns = snapshot.threads.flatMap((thread) => thread.turns ?? []);
  if (!turns.length) {
    return null;
  }

  return turns.reduce<TurnRuntime | null>((latest, turn) => {
    if (!latest) {
      return turn;
    }

    return getTurnTimestamp(turn) >= getTurnTimestamp(latest) ? turn : latest;
  }, null);
}

export function applyRuntimeSnapshot(
  current: SessionExecutionRuntimeInfo | null,
  snapshot?: SessionRuntimeSnapshot | null
): SessionExecutionRuntimeInfo | null {
  const latestTurn = selectLatestTurnRuntime(snapshot);
  if (!latestTurn) {
    return current;
  }

  return buildRuntimeInfo(
    current,
    {
      model:
        latestTurn.outputSchemaRuntime?.modelName ?? latestTurn.contextOverride?.model ?? undefined,
      provider: latestTurn.outputSchemaRuntime?.providerName ?? undefined,
      outputSchemaRuntime: latestTurn.outputSchemaRuntime ?? undefined,
    },
    'runtime_snapshot'
  );
}

export function applyTurnContextEvent(
  current: SessionExecutionRuntimeInfo | null,
  event: TurnContextEvent
): SessionExecutionRuntimeInfo | null {
  return buildRuntimeInfo(
    current,
    {
      model: event.output_schema_runtime?.modelName ?? undefined,
      provider: event.output_schema_runtime?.providerName ?? undefined,
      outputSchemaRuntime: event.output_schema_runtime ?? undefined,
    },
    'turn_context'
  );
}

export function applyModelChangeEvent(
  current: SessionExecutionRuntimeInfo | null,
  event: ModelChangeEvent
): SessionExecutionRuntimeInfo | null {
  return buildRuntimeInfo(
    current,
    {
      model: event.model,
      mode: event.mode,
    },
    'model_change'
  );
}

export function getOutputSchemaRuntimeLabel(
  runtime?: TurnOutputSchemaRuntime | null
): string | null {
  if (!runtime) {
    return null;
  }

  const strategyLabel = runtime.strategy === 'native' ? 'Native schema' : 'Final output tool';
  const sourceLabel = runtime.source === 'turn' ? 'turn contract' : 'session contract';
  return `${strategyLabel} · ${sourceLabel}`;
}
