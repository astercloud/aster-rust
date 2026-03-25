import { describe, expect, it } from 'vitest';
import type { MessageEvent, SessionRuntimeSnapshot } from '../api';
import {
  applyModelChangeEvent,
  applyRuntimeSnapshot,
  applyTurnContextEvent,
  createRuntimeInfoFromSession,
  getOutputSchemaRuntimeLabel,
  selectLatestTurnRuntime,
} from './sessionExecutionRuntime';

describe('sessionExecutionRuntime', () => {
  it('creates runtime info from the persisted session model/provider', () => {
    const result = createRuntimeInfoFromSession({
      id: 'session-1',
      created_at: '2026-03-24T00:00:00Z',
      extension_data: {},
      message_count: 0,
      model_config: {
        model_name: 'gpt-5.4',
        toolshim: false,
      },
      name: 'Test Session',
      provider_name: 'openai',
      updated_at: '2026-03-24T00:00:00Z',
      user_set_name: false,
      working_dir: '/tmp/workspace',
    });

    expect(result).toEqual({
      model: 'gpt-5.4',
      mode: null,
      outputSchemaRuntime: null,
      provider: 'openai',
      source: 'session',
    });
  });

  it('selects the newest turn across all threads', () => {
    const snapshot: SessionRuntimeSnapshot = {
      sessionId: 'session-1',
      threads: [
        {
          items: [],
          thread: {
            createdAt: '2026-03-24T00:00:00Z',
            id: 'thread-1',
            metadata: {},
            sessionId: 'session-1',
            status: 'active',
            updatedAt: '2026-03-24T00:00:00Z',
            workingDir: '/tmp/workspace',
          },
          turns: [
            {
              createdAt: '2026-03-24T00:00:00Z',
              id: 'turn-1',
              sessionId: 'session-1',
              status: 'completed',
              threadId: 'thread-1',
              updatedAt: '2026-03-24T00:00:01Z',
            },
          ],
        },
        {
          items: [],
          thread: {
            createdAt: '2026-03-24T00:00:00Z',
            id: 'thread-2',
            metadata: {},
            sessionId: 'session-1',
            status: 'active',
            updatedAt: '2026-03-24T00:00:00Z',
            workingDir: '/tmp/workspace',
          },
          turns: [
            {
              createdAt: '2026-03-24T00:00:00Z',
              id: 'turn-2',
              outputSchemaRuntime: {
                modelName: 'gpt-5.4',
                providerName: 'openai',
                source: 'turn',
                strategy: 'native',
              },
              sessionId: 'session-1',
              status: 'completed',
              threadId: 'thread-2',
              updatedAt: '2026-03-24T00:00:03Z',
            },
          ],
        },
      ],
    };

    expect(selectLatestTurnRuntime(snapshot)?.id).toBe('turn-2');
  });

  it('prefers runtime snapshot output schema provider/model over the persisted session config', () => {
    const current = createRuntimeInfoFromSession({
      id: 'session-1',
      created_at: '2026-03-24T00:00:00Z',
      extension_data: {},
      message_count: 0,
      model_config: {
        model_name: 'gpt-4.1',
        toolshim: false,
      },
      name: 'Test Session',
      provider_name: 'openai',
      updated_at: '2026-03-24T00:00:00Z',
      user_set_name: false,
      working_dir: '/tmp/workspace',
    });
    const snapshot: SessionRuntimeSnapshot = {
      sessionId: 'session-1',
      threads: [
        {
          items: [],
          thread: {
            createdAt: '2026-03-24T00:00:00Z',
            id: 'thread-1',
            metadata: {},
            sessionId: 'session-1',
            status: 'active',
            updatedAt: '2026-03-24T00:00:00Z',
            workingDir: '/tmp/workspace',
          },
          turns: [
            {
              contextOverride: {
                model: 'o3',
              },
              createdAt: '2026-03-24T00:00:00Z',
              id: 'turn-2',
              outputSchemaRuntime: {
                modelName: 'gpt-5.4',
                providerName: 'codex_app_server',
                source: 'turn',
                strategy: 'native',
              },
              sessionId: 'session-1',
              status: 'completed',
              threadId: 'thread-1',
              updatedAt: '2026-03-24T00:00:03Z',
            },
          ],
        },
      ],
    };

    expect(applyRuntimeSnapshot(current, snapshot)).toEqual({
      model: 'gpt-5.4',
      mode: null,
      outputSchemaRuntime: {
        modelName: 'gpt-5.4',
        providerName: 'codex_app_server',
        source: 'turn',
        strategy: 'native',
      },
      provider: 'codex_app_server',
      source: 'runtime_snapshot',
    });
  });

  it('applies turn context and model change events incrementally', () => {
    const turnContextEvent: Extract<MessageEvent, { type: 'TurnContext' }> = {
      output_schema_runtime: {
        modelName: 'gpt-5.4',
        providerName: 'codex_app_server',
        source: 'turn',
        strategy: 'native',
      },
      session_id: 'session-1',
      thread_id: 'thread-1',
      turn_id: 'turn-1',
      type: 'TurnContext',
    };
    const modelChangeEvent: Extract<MessageEvent, { type: 'ModelChange' }> = {
      mode: 'worker',
      model: 'o3',
      type: 'ModelChange',
    };

    const afterTurnContext = applyTurnContextEvent(null, turnContextEvent);
    expect(afterTurnContext).toEqual({
      model: 'gpt-5.4',
      mode: null,
      outputSchemaRuntime: turnContextEvent.output_schema_runtime,
      provider: 'codex_app_server',
      source: 'turn_context',
    });

    const afterModelChange = applyModelChangeEvent(afterTurnContext, modelChangeEvent);
    expect(afterModelChange).toEqual({
      model: 'o3',
      mode: 'worker',
      outputSchemaRuntime: turnContextEvent.output_schema_runtime,
      provider: 'codex_app_server',
      source: 'model_change',
    });
  });

  it('formats the schema runtime label for dropdown display', () => {
    expect(
      getOutputSchemaRuntimeLabel({
        source: 'session',
        strategy: 'final_output_tool',
      })
    ).toBe('Final output tool · session contract');
  });
});
