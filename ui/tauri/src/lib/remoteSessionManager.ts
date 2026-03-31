const DEFAULT_BASE_URL = "http://127.0.0.1:3000";
const DEFAULT_SECRET_KEY = "test";

export interface RemoteSessionMessage {
  id: string;
  role: string;
  content: string;
  timestamp: string;
}

export type RemoteSessionActivityKind =
  | "tool_call"
  | "approval_request"
  | "request_user_input"
  | "runtime_status"
  | "file_artifact"
  | "model_change"
  | "context_compaction";

export type RemoteSessionActivityState =
  | "started"
  | "updated"
  | "completed"
  | "failed"
  | "info";

export interface RemoteSessionActivity {
  id: string;
  kind: RemoteSessionActivityKind;
  state: RemoteSessionActivityState;
  title: string;
  detail: string;
  timestamp: string;
}

export type RemoteSessionToolConfirmationAction =
  | "always_allow"
  | "allow_once"
  | "deny";

interface SessionApiMessageContent {
  type: string;
  text?: string;
  toolName?: string;
  tool_name?: string;
}

interface SessionApiMessage {
  id?: string | null;
  role: "user" | "assistant";
  created: number;
  content: SessionApiMessageContent[];
  metadata?: {
    userVisible?: boolean;
  };
}

interface SessionApiResponse {
  conversation?: SessionApiMessage[] | null;
}

interface RemoteSessionReplyRequest {
  user_message: {
    id: string;
    role: "user";
    created: number;
    content: Array<{
      type: "text";
      text: string;
      meta: null;
    }>;
    metadata: {
      userVisible: true;
      agentVisible: true;
    };
  };
  session_id: string;
}

interface RemoteSessionToolConfirmationRequest {
  id: string;
  principal_type: "tool";
  action: RemoteSessionToolConfirmationAction;
  session_id: string;
}

interface RemoteSessionManagerOptions {
  sessionId: string;
  baseUrl?: string;
  secretKey?: string;
  pollIntervalMs?: number;
  onMessages?: (messages: RemoteSessionMessage[]) => void;
  onActivities?: (activities: RemoteSessionActivity[]) => void;
  onStatus?: (status: string | null) => void;
  onError?: (error: Error) => void;
}

interface ReplyStreamMessageEvent {
  type: "Message";
  message: SessionApiMessage;
}

interface ReplyStreamUpdateConversationEvent {
  type: "UpdateConversation";
  conversation?: SessionApiMessage[] | { messages?: SessionApiMessage[] } | null;
}

interface ReplyStreamErrorEvent {
  type: "Error";
  error: string;
}

interface ReplyStreamFinishEvent {
  type: "Finish";
  reason: string;
}

interface ReplyStreamPingEvent {
  type: "Ping";
}

interface ReplyStreamTurnContextEvent {
  type: "TurnContext";
  turn_id?: string;
}

interface ReplyStreamItemEvent {
  type: "ItemStarted" | "ItemUpdated" | "ItemCompleted";
  item: ReplyStreamRuntimeItem;
}

interface ReplyStreamModelChangeEvent {
  type: "ModelChange";
  model: string;
  mode: string;
}

interface ReplyStreamContextCompactionEvent {
  type: "ContextCompactionStarted" | "ContextCompactionCompleted";
  item_id: string;
  trigger?: string;
  detail?: string | null;
}

interface ReplyStreamContextCompactionWarningEvent {
  type: "ContextCompactionWarning";
  message: string;
}

interface ReplyStreamRuntimeItem {
  id: string;
  type: string;
  status?: string;
  startedAt?: string;
  completedAt?: string | null;
  updatedAt?: string;
  content?: string;
  text?: string;
  summary?: string[];
  phase?: string;
  title?: string;
  detail?: string;
  checkpoints?: string[];
  path?: string;
  source?: string;
  tool_name?: string;
  success?: boolean;
  error?: string;
  action_type?: string;
  prompt?: string;
  request_id?: string;
}

type ReplyStreamEvent =
  | ReplyStreamMessageEvent
  | ReplyStreamUpdateConversationEvent
  | ReplyStreamErrorEvent
  | ReplyStreamFinishEvent
  | ReplyStreamPingEvent
  | ReplyStreamTurnContextEvent
  | ReplyStreamItemEvent
  | ReplyStreamModelChangeEvent
  | ReplyStreamContextCompactionEvent
  | ReplyStreamContextCompactionWarningEvent
  | { type: string };

function isReplyStreamErrorEvent(
  event: ReplyStreamEvent,
): event is ReplyStreamErrorEvent {
  return event.type === "Error";
}

function isReplyStreamUpdateConversationEvent(
  event: ReplyStreamEvent,
): event is ReplyStreamUpdateConversationEvent {
  return event.type === "UpdateConversation";
}

function isReplyStreamMessageEvent(
  event: ReplyStreamEvent,
): event is ReplyStreamMessageEvent {
  return event.type === "Message";
}

function isReplyStreamTurnContextEvent(
  event: ReplyStreamEvent,
): event is ReplyStreamTurnContextEvent {
  return event.type === "TurnContext";
}

function isReplyStreamItemEvent(
  event: ReplyStreamEvent,
): event is ReplyStreamItemEvent {
  return (
    event.type === "ItemStarted" ||
    event.type === "ItemUpdated" ||
    event.type === "ItemCompleted"
  );
}

function isReplyStreamModelChangeEvent(
  event: ReplyStreamEvent,
): event is ReplyStreamModelChangeEvent {
  return event.type === "ModelChange";
}

function isReplyStreamContextCompactionEvent(
  event: ReplyStreamEvent,
): event is ReplyStreamContextCompactionEvent {
  return (
    event.type === "ContextCompactionStarted" ||
    event.type === "ContextCompactionCompleted"
  );
}

function isReplyStreamContextCompactionWarningEvent(
  event: ReplyStreamEvent,
): event is ReplyStreamContextCompactionWarningEvent {
  return event.type === "ContextCompactionWarning";
}

function buildHeaders(secretKey: string): HeadersInit {
  return {
    "Content-Type": "application/json",
    "X-Secret-Key": secretKey,
  };
}

function renderMessageContent(content: SessionApiMessageContent[]): string {
  return content
    .map((item) => {
      if (item.type === "text") {
        return item.text ?? "";
      }
      const toolName = item.toolName ?? item.tool_name;
      if (toolName) {
        return `[${toolName}]`;
      }
      return `[${item.type}]`;
    })
    .filter((item) => item.trim().length > 0)
    .join("\n");
}

function toRemoteMessage(message: SessionApiMessage): RemoteSessionMessage | null {
  if (message.metadata?.userVisible === false) {
    return null;
  }

  return {
    id: message.id ?? `msg-${message.created}`,
    role: message.role,
    content: renderMessageContent(message.content),
    timestamp: new Date(message.created * 1000).toISOString(),
  };
}

function parseSessionMessages(payload: SessionApiResponse): RemoteSessionMessage[] {
  const messages = payload.conversation ?? [];
  return messages
    .map(toRemoteMessage)
    .filter((message): message is RemoteSessionMessage => Boolean(message));
}

function mergeStreamingAssistantMessage(
  current: RemoteSessionMessage | null,
  next: RemoteSessionMessage,
): RemoteSessionMessage {
  if (!current || current.id !== next.id) {
    return next;
  }

  const mergedContent = next.content.startsWith(current.content)
    ? next.content
    : `${current.content}${next.content}`;

  return {
    ...next,
    content: mergedContent,
  };
}

function extractSseDataBlocks(chunk: string): { blocks: string[]; remainder: string } {
  const normalized = chunk.replace(/\r\n/g, "\n");
  const segments = normalized.split("\n\n");
  const remainder = segments.pop() ?? "";
  return {
    blocks: segments,
    remainder,
  };
}

function parseSseBlock(block: string): ReplyStreamEvent | null {
  const data = block
    .split("\n")
    .filter((line) => line.startsWith("data:"))
    .map((line) => line.slice(5).trimStart())
    .join("\n")
    .trim();

  if (!data) {
    return null;
  }

  try {
    return JSON.parse(data) as ReplyStreamEvent;
  } catch {
    return null;
  }
}

function parseConversationUpdate(
  event: ReplyStreamUpdateConversationEvent,
): RemoteSessionMessage[] | null {
  if (Array.isArray(event.conversation)) {
    return parseSessionMessages({
      conversation: event.conversation,
    });
  }

  if (
    event.conversation &&
    typeof event.conversation === "object" &&
    Array.isArray(event.conversation.messages)
  ) {
    return parseSessionMessages({
      conversation: event.conversation.messages,
    });
  }

  return null;
}

function truncateStatus(value: string, maxLength = 96): string {
  const normalized = value.replace(/\s+/g, " ").trim();
  if (normalized.length <= maxLength) {
    return normalized;
  }
  return `${normalized.slice(0, maxLength - 1).trimEnd()}…`;
}

function resolveActivityState(
  eventType: ReplyStreamItemEvent["type"],
  item: ReplyStreamRuntimeItem,
): RemoteSessionActivityState {
  if (item.status === "failed" || item.success === false) {
    return "failed";
  }

  if (eventType === "ItemCompleted" || item.status === "completed") {
    return "completed";
  }

  if (eventType === "ItemStarted") {
    return "started";
  }

  return "updated";
}

function resolveActivityTimestamp(item: ReplyStreamRuntimeItem): string {
  return (
    item.updatedAt ??
    item.completedAt ??
    item.startedAt ??
    new Date().toISOString()
  );
}

function buildToolCallActivity(
  eventType: ReplyStreamItemEvent["type"],
  item: ReplyStreamRuntimeItem,
): RemoteSessionActivity {
  const state = resolveActivityState(eventType, item);
  const title = item.tool_name ? `工具 ${item.tool_name}` : "工具调用";
  let detail = "执行中";

  if (state === "started") {
    detail = "开始执行";
  } else if (state === "completed") {
    detail = "执行完成";
  } else if (state === "failed") {
    detail = truncateStatus(item.error ?? "执行失败");
  }

  return {
    id: item.id,
    kind: "tool_call",
    state,
    title,
    detail,
    timestamp: resolveActivityTimestamp(item),
  };
}

function buildRuntimeStatusActivity(
  eventType: ReplyStreamItemEvent["type"],
  item: ReplyStreamRuntimeItem,
): RemoteSessionActivity {
  const state = resolveActivityState(eventType, item);
  const title = truncateStatus(item.title ?? item.phase ?? "运行状态");
  const detailSource =
    item.detail ?? item.checkpoints?.[0] ?? (state === "started" ? "开始执行" : "处理中");

  return {
    id: item.id,
    kind: "runtime_status",
    state,
    title,
    detail: truncateStatus(detailSource),
    timestamp: resolveActivityTimestamp(item),
  };
}

function buildApprovalActivity(
  eventType: ReplyStreamItemEvent["type"],
  item: ReplyStreamRuntimeItem,
): RemoteSessionActivity {
  const state = resolveActivityState(eventType, item);
  const title = item.tool_name
    ? `等待批准 ${item.tool_name}`
    : item.action_type
      ? `等待批准 ${item.action_type}`
      : "等待批准";
  const detail = truncateStatus(item.prompt ?? "需要确认后继续");

  return {
    id: item.request_id ?? item.id,
    kind: "approval_request",
    state,
    title,
    detail,
    timestamp: resolveActivityTimestamp(item),
  };
}

function buildRequestInputActivity(
  eventType: ReplyStreamItemEvent["type"],
  item: ReplyStreamRuntimeItem,
): RemoteSessionActivity {
  const state = resolveActivityState(eventType, item);
  const title = item.action_type
    ? `等待输入 ${item.action_type}`
    : "等待用户输入";
  const detail = truncateStatus(item.prompt ?? "需要补充信息");

  return {
    id: item.request_id ?? item.id,
    kind: "request_user_input",
    state,
    title,
    detail,
    timestamp: resolveActivityTimestamp(item),
  };
}

function buildFileArtifactActivity(
  eventType: ReplyStreamItemEvent["type"],
  item: ReplyStreamRuntimeItem,
): RemoteSessionActivity {
  const state = resolveActivityState(eventType, item);
  const title = item.path ? `生成文件 ${item.path}` : "生成文件";
  const detail = truncateStatus(item.source ? `来源：${item.source}` : "文件产物已更新");

  return {
    id: item.id,
    kind: "file_artifact",
    state,
    title,
    detail,
    timestamp: resolveActivityTimestamp(item),
  };
}

function buildActivityFromRuntimeItem(
  event: ReplyStreamItemEvent,
): RemoteSessionActivity | null {
  switch (event.item.type) {
    case "tool_call":
      return buildToolCallActivity(event.type, event.item);
    case "approval_request":
      return buildApprovalActivity(event.type, event.item);
    case "request_user_input":
      return buildRequestInputActivity(event.type, event.item);
    case "runtime_status":
      return buildRuntimeStatusActivity(event.type, event.item);
    case "file_artifact":
      return buildFileArtifactActivity(event.type, event.item);
    default:
      return null;
  }
}

function formatActivityStatus(state: RemoteSessionActivityState): string {
  switch (state) {
    case "started":
      return "开始";
    case "updated":
      return "进行中";
    case "completed":
      return "完成";
    case "failed":
      return "失败";
    case "info":
      return "更新";
  }
}

function mergeActivity(
  current: RemoteSessionActivity[],
  next: RemoteSessionActivity,
  maxLength = 6,
): RemoteSessionActivity[] {
  const remaining = current.filter((item) => item.id !== next.id);
  return [...remaining, next].slice(-maxLength);
}

function summarizeReplyStreamStatus(event: ReplyStreamEvent): string | null {
  if (isReplyStreamTurnContextEvent(event)) {
    return "已开始生成回复";
  }

  if (isReplyStreamItemEvent(event)) {
    const activity = buildActivityFromRuntimeItem(event);
    if (activity) {
      return `${formatActivityStatus(activity.state)}：${truncateStatus(activity.title)}`;
    }
    return `进行中：${truncateStatus(event.item.type)}`;
  }

  if (isReplyStreamModelChangeEvent(event)) {
    return `切换模型：${event.model}`;
  }

  if (isReplyStreamContextCompactionEvent(event)) {
    return event.type === "ContextCompactionStarted"
      ? "正在压缩上下文"
      : "上下文压缩完成";
  }

  if (isReplyStreamContextCompactionWarningEvent(event)) {
    return truncateStatus(event.message);
  }

  if (event.type === "Finish") {
    return "响应完成";
  }

  return null;
}

function buildActivityFromEvent(
  event: ReplyStreamEvent,
): RemoteSessionActivity | null {
  if (isReplyStreamItemEvent(event)) {
    return buildActivityFromRuntimeItem(event);
  }

  if (isReplyStreamModelChangeEvent(event)) {
    return {
      id: `model-change-${Date.now()}`,
      kind: "model_change",
      state: "info",
      title: `切换模型 ${event.model}`,
      detail: truncateStatus(`模式：${event.mode}`),
      timestamp: new Date().toISOString(),
    };
  }

  if (isReplyStreamContextCompactionEvent(event)) {
    return {
      id: `context-compaction-${event.item_id}`,
      kind: "context_compaction",
      state:
        event.type === "ContextCompactionStarted" ? "started" : "completed",
      title: "上下文压缩",
      detail: truncateStatus(event.detail ?? event.trigger ?? "上下文已更新"),
      timestamp: new Date().toISOString(),
    };
  }

  if (isReplyStreamContextCompactionWarningEvent(event)) {
    return {
      id: `context-warning-${Date.now()}`,
      kind: "context_compaction",
      state: "failed",
      title: "上下文压缩警告",
      detail: truncateStatus(event.message),
      timestamp: new Date().toISOString(),
    };
  }

  return null;
}

function describeToolConfirmationAction(
  action: RemoteSessionToolConfirmationAction,
): string {
  switch (action) {
    case "allow_once":
      return "本次允许";
    case "always_allow":
      return "始终允许";
    case "deny":
      return "拒绝";
  }
}

export class RemoteSessionManager {
  private readonly sessionId: string;
  private readonly baseUrl: string;
  private readonly secretKey: string;
  private readonly pollIntervalMs: number;
  private readonly onMessages?: (messages: RemoteSessionMessage[]) => void;
  private readonly onActivities?: (activities: RemoteSessionActivity[]) => void;
  private readonly onStatus?: (status: string | null) => void;
  private readonly onError?: (error: Error) => void;

  private pollTimer: number | null = null;
  private refreshAbortController: AbortController | null = null;
  private sendAbortController: AbortController | null = null;
  private latestMessages: RemoteSessionMessage[] = [];
  private latestActivities: RemoteSessionActivity[] = [];
  private latestStatus: string | null = null;

  constructor(options: RemoteSessionManagerOptions) {
    this.sessionId = options.sessionId;
    this.baseUrl = options.baseUrl ?? DEFAULT_BASE_URL;
    this.secretKey = options.secretKey ?? DEFAULT_SECRET_KEY;
    this.pollIntervalMs = options.pollIntervalMs ?? 2000;
    this.onMessages = options.onMessages;
    this.onActivities = options.onActivities;
    this.onStatus = options.onStatus;
    this.onError = options.onError;
  }

  private publishStatus(status: string | null): void {
    if (this.latestStatus === status) {
      return;
    }

    this.latestStatus = status;
    this.onStatus?.(status);
  }

  private resetActivities(): void {
    this.latestActivities = [];
    this.onActivities?.([]);
  }

  private publishActivity(activity: RemoteSessionActivity | null): void {
    if (!activity) {
      return;
    }

    this.latestActivities = mergeActivity(this.latestActivities, activity);
    this.onActivities?.(this.latestActivities);
  }

  private patchActivity(
    activityId: string,
    updater: (activity: RemoteSessionActivity) => RemoteSessionActivity,
  ): void {
    let changed = false;
    this.latestActivities = this.latestActivities.map((activity) => {
      if (activity.id !== activityId) {
        return activity;
      }
      changed = true;
      return updater(activity);
    });

    if (changed) {
      this.onActivities?.(this.latestActivities);
    }
  }

  async connect(): Promise<void> {
    await this.refreshMessages();

    if (this.pollTimer !== null) {
      window.clearInterval(this.pollTimer);
    }

    this.pollTimer = window.setInterval(() => {
      void this.refreshMessages();
    }, this.pollIntervalMs);
  }

  disconnect(): void {
    if (this.pollTimer !== null) {
      window.clearInterval(this.pollTimer);
      this.pollTimer = null;
    }

    this.refreshAbortController?.abort();
    this.refreshAbortController = null;

    this.sendAbortController?.abort();
    this.sendAbortController = null;
    this.publishStatus(null);
    this.resetActivities();
  }

  async refreshMessages(): Promise<RemoteSessionMessage[]> {
    this.refreshAbortController?.abort();
    const controller = new AbortController();
    this.refreshAbortController = controller;

    try {
      const response = await fetch(
        `${this.baseUrl}/sessions/${encodeURIComponent(this.sessionId)}`,
        {
          headers: buildHeaders(this.secretKey),
          signal: controller.signal,
        },
      );

      if (!response.ok) {
        throw new Error(`拉取会话失败: ${response.status}`);
      }

      const payload = (await response.json()) as SessionApiResponse;
      const messages = parseSessionMessages(payload);
      this.latestMessages = messages;
      this.onMessages?.(messages);
      return messages;
    } catch (error) {
      if ((error as Error).name !== "AbortError") {
        this.onError?.(error as Error);
      }
      throw error;
    }
  }

  private publishStreamingAssistant(message: RemoteSessionMessage): void {
    const baseMessages = this.latestMessages.filter((item) => item.id !== message.id);
    this.onMessages?.([...baseMessages, message]);
  }

  private publishOptimisticUserMessage(message: RemoteSessionMessage): void {
    this.latestMessages = [
      ...this.latestMessages.filter((item) => item.id !== message.id),
      message,
    ];
    this.onMessages?.(this.latestMessages);
  }

  async sendMessage(
    content: string,
    optimisticUserMessage?: RemoteSessionMessage,
  ): Promise<void> {
    this.sendAbortController?.abort();
    const controller = new AbortController();
    this.sendAbortController = controller;

    const payload: RemoteSessionReplyRequest = {
      session_id: this.sessionId,
      user_message: {
        id: `user-${Date.now()}`,
        role: "user",
        created: Math.floor(Date.now() / 1000),
        content: [
          {
            type: "text",
            text: content,
            meta: null,
          },
        ],
        metadata: {
          userVisible: true,
          agentVisible: true,
        },
      },
    };

    if (optimisticUserMessage) {
      this.publishOptimisticUserMessage(optimisticUserMessage);
    }
    this.resetActivities();
    this.publishStatus("等待响应...");

    try {
      const response = await fetch(`${this.baseUrl}/reply`, {
        method: "POST",
        headers: buildHeaders(this.secretKey),
        body: JSON.stringify(payload),
        signal: controller.signal,
      });

      if (!response.ok) {
        throw new Error(`发送消息失败: ${response.status}`);
      }

      if (response.body) {
        const reader = response.body.getReader();
        const decoder = new TextDecoder();
        let buffer = "";
        let streamingAssistant: RemoteSessionMessage | null = null;

        while (true) {
          const { done, value } = await reader.read();
          if (done) {
            break;
          }

          buffer += decoder.decode(value, { stream: true });
          const { blocks, remainder } = extractSseDataBlocks(buffer);
          buffer = remainder;

          for (const block of blocks) {
            const event = parseSseBlock(block);
            if (!event || event.type === "Ping") {
              continue;
            }

            if (isReplyStreamErrorEvent(event)) {
              throw new Error(event.error);
            }

            if (isReplyStreamUpdateConversationEvent(event)) {
              const nextMessages = parseConversationUpdate(event);
              if (nextMessages) {
                this.latestMessages = nextMessages;
                this.onMessages?.(nextMessages);
              }
              continue;
            }

            const status = summarizeReplyStreamStatus(event);
            if (status) {
              this.publishStatus(status);
            }

            this.publishActivity(buildActivityFromEvent(event));

            if (isReplyStreamMessageEvent(event)) {
              const remoteMessage = toRemoteMessage(event.message);
              if (!remoteMessage || remoteMessage.role !== "assistant") {
                continue;
              }

              if (streamingAssistant === null) {
                this.publishStatus("生成回复中...");
              }
              streamingAssistant = mergeStreamingAssistantMessage(
                streamingAssistant,
                remoteMessage,
              );
              this.publishStreamingAssistant(streamingAssistant);
            }
          }
        }

        const trailingEvent = parseSseBlock(buffer);
        if (trailingEvent && isReplyStreamMessageEvent(trailingEvent)) {
          const remoteMessage = toRemoteMessage(trailingEvent.message);
          if (remoteMessage && remoteMessage.role === "assistant") {
            if (streamingAssistant === null) {
              this.publishStatus("生成回复中...");
            }
            streamingAssistant = mergeStreamingAssistantMessage(
              streamingAssistant,
              remoteMessage,
            );
            this.publishStreamingAssistant(streamingAssistant);
          }
        }
      }

      await this.refreshMessages();
    } catch (error) {
      if ((error as Error).name !== "AbortError") {
        this.publishStatus("响应失败");
        this.onMessages?.(this.latestMessages);
        this.onError?.(error as Error);
      }
      throw error;
    }
  }

  async confirmToolAction(
    requestId: string,
    action: RemoteSessionToolConfirmationAction,
  ): Promise<void> {
    const actionLabel = describeToolConfirmationAction(action);
    const payload: RemoteSessionToolConfirmationRequest = {
      id: requestId,
      principal_type: "tool",
      action,
      session_id: this.sessionId,
    };

    this.patchActivity(requestId, (activity) => ({
      ...activity,
      state: "info",
      detail: `已提交${actionLabel}，等待继续...`,
      timestamp: new Date().toISOString(),
    }));
    this.publishStatus(`已提交${actionLabel}`);

    try {
      const response = await fetch(
        `${this.baseUrl}/action-required/tool-confirmation`,
        {
          method: "POST",
          headers: buildHeaders(this.secretKey),
          body: JSON.stringify(payload),
        },
      );

      if (!response.ok) {
        throw new Error(`提交批准失败: ${response.status}`);
      }
    } catch (error) {
      this.patchActivity(requestId, (activity) => ({
        ...activity,
        state: "failed",
        detail: truncateStatus(
          (error as Error).message || `提交${actionLabel}失败`,
        ),
        timestamp: new Date().toISOString(),
      }));
      this.publishStatus("提交批准失败");
      this.onError?.(error as Error);
      throw error;
    }
  }
}
