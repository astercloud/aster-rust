import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  RemoteSessionManager,
  type RemoteSessionActivity,
  type RemoteSessionMessage as Message,
  type RemoteSessionToolConfirmationAction,
} from "../lib/remoteSessionManager";

interface ChatProps {
  sessionId: string;
}

function getActivityStateLabel(state: RemoteSessionActivity["state"]): string {
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

function getActivityStateClass(state: RemoteSessionActivity["state"]): string {
  switch (state) {
    case "started":
      return "border-blue-500/30 bg-blue-500/10 text-blue-200";
    case "updated":
      return "border-amber-500/30 bg-amber-500/10 text-amber-200";
    case "completed":
      return "border-emerald-500/30 bg-emerald-500/10 text-emerald-200";
    case "failed":
      return "border-red-500/30 bg-red-500/10 text-red-200";
    case "info":
      return "border-slate-500/30 bg-slate-500/10 text-slate-200";
  }
}

function getActivityKindLabel(kind: RemoteSessionActivity["kind"]): string {
  switch (kind) {
    case "tool_call":
      return "工具";
    case "approval_request":
      return "批准";
    case "request_user_input":
      return "输入";
    case "runtime_status":
      return "状态";
    case "file_artifact":
      return "文件";
    case "model_change":
      return "模型";
    case "context_compaction":
      return "上下文";
  }
}

function formatActivityTime(timestamp: string): string {
  const parsed = new Date(timestamp);
  if (Number.isNaN(parsed.getTime())) {
    return "";
  }

  return parsed.toLocaleTimeString("zh-CN", {
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
  });
}

export default function Chat({ sessionId }: ChatProps) {
  const [messages, setMessages] = useState<Message[]>([]);
  const [activities, setActivities] = useState<RemoteSessionActivity[]>([]);
  const [input, setInput] = useState("");
  const [loading, setLoading] = useState(false);
  const [pendingApprovalId, setPendingApprovalId] = useState<string | null>(null);
  const [runtimeStatus, setRuntimeStatus] = useState<string | null>(null);
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const remoteManagerRef = useRef<RemoteSessionManager | null>(null);

  useEffect(() => {
    setRuntimeStatus(null);
    setActivities([]);
    setPendingApprovalId(null);
    const manager = new RemoteSessionManager({
      sessionId,
      onMessages: setMessages,
      onActivities: setActivities,
      onStatus: setRuntimeStatus,
      onError: (error) => {
        console.error("Remote session refresh failed:", error);
      },
    });

    remoteManagerRef.current = manager;

    manager.connect().catch(async () => {
      await loadMessages();
    });

    return () => {
      manager.disconnect();
      if (remoteManagerRef.current === manager) {
        remoteManagerRef.current = null;
      }
    };
  }, [sessionId]);

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages]);


  async function loadMessages() {
    try {
      const result = await invoke<Message[]>("get_session_messages", {
        sessionId,
      });
      setMessages(result);
    } catch (error) {
      console.error("Failed to load messages:", error);
    }
  }

  async function sendMessage() {
    if (!input.trim() || loading) return;

    const content = input.trim();
    const optimisticMessage: Message = {
      id: `local-${Date.now()}`,
      role: "user",
      content,
      timestamp: new Date().toISOString(),
    };

    setLoading(true);
    setMessages((prev) => [...prev, optimisticMessage]);
    setInput("");

    try {
      if (remoteManagerRef.current) {
        await remoteManagerRef.current.sendMessage(content, optimisticMessage);
      } else {
        setRuntimeStatus("本地模式已保存消息");
        const message = await invoke<Message>("send_message", {
          sessionId,
          content,
        });
        setMessages((prev) => [
          ...prev.filter((item) => item.id !== optimisticMessage.id),
          message,
        ]);
      }
    } catch (error) {
      console.error("Failed to send message:", error);
      try {
        const message = await invoke<Message>("send_message", {
          sessionId,
          content,
        });
        setMessages((prev) => [
          ...prev.filter((item) => item.id !== optimisticMessage.id),
          message,
        ]);
      } catch (fallbackError) {
        console.error("Failed to persist local message:", fallbackError);
        setMessages((prev) =>
          prev.filter((item) => item.id !== optimisticMessage.id),
        );
      }
    } finally {
      setLoading(false);
    }
  }

  async function handleApprovalAction(
    activityId: string,
    action: RemoteSessionToolConfirmationAction,
  ) {
    if (!remoteManagerRef.current || pendingApprovalId) {
      return;
    }

    setPendingApprovalId(activityId);
    try {
      await remoteManagerRef.current.confirmToolAction(activityId, action);
    } catch (error) {
      console.error("Failed to confirm tool action:", error);
    } finally {
      setPendingApprovalId(null);
    }
  }


  return (
    <div className="flex-1 flex flex-col">
      <div className="flex-1 overflow-y-auto p-4 space-y-4">
        {messages.map((msg) => (
          <div
            key={msg.id}
            className={`p-3 rounded-lg ${
              msg.role === "user"
                ? "bg-blue-600 ml-auto max-w-[80%]"
                : "bg-gray-700 mr-auto max-w-[80%]"
            }`}
          >
            <div className="whitespace-pre-wrap">{msg.content}</div>
          </div>
        ))}
        <div ref={messagesEndRef} />
      </div>

      {activities.length > 0 ? (
        <div className="border-t border-gray-800 bg-gray-950/80 px-4 py-3">
          <div className="mb-3 flex items-center justify-between text-xs text-gray-400">
            <span>最近活动</span>
            <span>{activities.length} 条</span>
          </div>
          <div className="space-y-2">
            {[...activities].reverse().map((activity) => (
              <div
                key={activity.id}
                className="rounded-lg border border-gray-800 bg-gray-900/80 px-3 py-2"
              >
                <div className="flex items-start justify-between gap-3">
                  <div className="min-w-0 flex-1">
                    <div className="flex items-center gap-2 text-xs text-gray-500">
                      <span>{getActivityKindLabel(activity.kind)}</span>
                      <span>{formatActivityTime(activity.timestamp)}</span>
                    </div>
                    <div className="mt-1 text-sm font-medium text-gray-100">
                      {activity.title}
                    </div>
                    <div className="mt-1 whitespace-pre-wrap text-xs text-gray-400">
                      {activity.detail}
                    </div>
                    {activity.kind === "approval_request" &&
                    activity.state !== "completed" &&
                    activity.state !== "failed" ? (
                      <div className="mt-3 flex flex-wrap gap-2">
                        <button
                          onClick={() =>
                            handleApprovalAction(activity.id, "deny")
                          }
                          disabled={pendingApprovalId !== null}
                          className="rounded border border-red-500/30 bg-red-500/10 px-2.5 py-1 text-[11px] text-red-200 disabled:opacity-50"
                        >
                          拒绝
                        </button>
                        <button
                          onClick={() =>
                            handleApprovalAction(activity.id, "allow_once")
                          }
                          disabled={pendingApprovalId !== null}
                          className="rounded border border-blue-500/30 bg-blue-500/10 px-2.5 py-1 text-[11px] text-blue-200 disabled:opacity-50"
                        >
                          本次允许
                        </button>
                        <button
                          onClick={() =>
                            handleApprovalAction(activity.id, "always_allow")
                          }
                          disabled={pendingApprovalId !== null}
                          className="rounded border border-emerald-500/30 bg-emerald-500/10 px-2.5 py-1 text-[11px] text-emerald-200 disabled:opacity-50"
                        >
                          始终允许
                        </button>
                      </div>
                    ) : null}
                  </div>
                  <span
                    className={`shrink-0 rounded-full border px-2 py-1 text-[11px] ${getActivityStateClass(activity.state)}`}
                  >
                    {pendingApprovalId === activity.id
                      ? "提交中"
                      : getActivityStateLabel(activity.state)}
                  </span>
                </div>
              </div>
            ))}
          </div>
        </div>
      ) : null}

      {runtimeStatus ? (
        <div className="px-4 py-2 text-sm text-blue-200 border-t border-gray-800 bg-gray-900/80">
          状态：{runtimeStatus}
        </div>
      ) : null}

      <div className="p-4 border-t border-gray-700">
        <div className="flex gap-2">
          <input
            type="text"
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && sendMessage()}
            placeholder="Type a message..."
            className="flex-1 px-4 py-2 bg-gray-700 rounded-lg"
            disabled={loading}
          />
          <button
            onClick={sendMessage}
            disabled={loading}
            className="px-6 py-2 bg-blue-600 rounded-lg hover:bg-blue-700 disabled:opacity-50"
          >
            Send
          </button>
        </div>
      </div>
    </div>
  );
}
