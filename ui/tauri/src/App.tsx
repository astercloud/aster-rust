import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import Chat from "./components/Chat";
import Sidebar from "./components/Sidebar";

interface SessionInfo {
  id: string;
  name: string;
  created_at: string;
  working_dir: string;
}

function App() {
  const [sessions, setSessions] = useState<SessionInfo[]>([]);
  const [currentSession, setCurrentSession] = useState<string | null>(null);
  const [serverStatus, setServerStatus] = useState<string>("Stopped");

  useEffect(() => {
    loadSessions();
    checkServerStatus();
  }, []);

  async function loadSessions() {
    try {
      const result = await invoke<SessionInfo[]>("get_sessions");
      setSessions(result);
    } catch (error) {
      console.error("Failed to load sessions:", error);
    }
  }

  async function checkServerStatus() {
    try {
      const status = await invoke<string>("get_server_status");
      setServerStatus(status);
    } catch (error) {
      console.error("Failed to get server status:", error);
    }
  }


  async function createSession() {
    try {
      const session = await invoke<SessionInfo>("start_session", {
        name: `Session ${sessions.length + 1}`,
        workingDir: ".",
      });
      setSessions([...sessions, session]);
      setCurrentSession(session.id);
    } catch (error) {
      console.error("Failed to create session:", error);
    }
  }

  return (
    <div className="flex h-screen bg-gray-900 text-white">
      <Sidebar
        sessions={sessions}
        currentSession={currentSession}
        onSelectSession={setCurrentSession}
        onNewSession={createSession}
        serverStatus={serverStatus}
      />
      <main className="flex-1 flex flex-col">
        {currentSession ? (
          <Chat sessionId={currentSession} />
        ) : (
          <div className="flex-1 flex items-center justify-center">
            <div className="text-center">
              <h1 className="text-3xl font-bold mb-4">Welcome to Aster</h1>
              <p className="text-gray-400 mb-6">
                Start a new session to begin
              </p>
              <button
                onClick={createSession}
                className="px-6 py-3 bg-blue-600 rounded-lg hover:bg-blue-700"
              >
                New Session
              </button>
            </div>
          </div>
        )}
      </main>
    </div>
  );
}

export default App;
