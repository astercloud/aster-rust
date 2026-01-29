interface SessionInfo {
  id: string;
  name: string;
  created_at: string;
  working_dir: string;
}

interface SidebarProps {
  sessions: SessionInfo[];
  currentSession: string | null;
  onSelectSession: (id: string) => void;
  onNewSession: () => void;
  serverStatus: string;
}

export default function Sidebar({
  sessions,
  currentSession,
  onSelectSession,
  onNewSession,
  serverStatus,
}: SidebarProps) {
  return (
    <aside className="w-64 bg-gray-800 border-r border-gray-700 flex flex-col">
      <div className="p-4 border-b border-gray-700">
        <h1 className="text-xl font-bold">Aster</h1>
        <div className="text-sm text-gray-400 mt-1">
          Server: {serverStatus}
        </div>
      </div>

      <div className="p-4">
        <button
          onClick={onNewSession}
          className="w-full px-4 py-2 bg-blue-600 rounded hover:bg-blue-700"
        >
          + New Session
        </button>
      </div>

      <div className="flex-1 overflow-y-auto">
        {sessions.map((session) => (
          <div
            key={session.id}
            onClick={() => onSelectSession(session.id)}
            className={`p-3 cursor-pointer hover:bg-gray-700 ${
              currentSession === session.id ? "bg-gray-700" : ""
            }`}
          >
            <div className="font-medium">{session.name}</div>
            <div className="text-sm text-gray-400 truncate">
              {session.working_dir}
            </div>
          </div>
        ))}
      </div>
    </aside>
  );
}
