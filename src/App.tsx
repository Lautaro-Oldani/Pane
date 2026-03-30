import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { initDatabase } from "./lib/db";
import "./App.css";

interface Clip {
  id: number;
  content: string;
  content_type: string;
  preview: string | null;
  image_base64: string | null;
  hash: string;
  is_pinned: boolean;
  is_favorite: boolean;
  created_at: string;
}

function App() {
  const [dbReady, setDbReady] = useState(false);
  const [clips, setClips] = useState<Clip[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [copiedId, setCopiedId] = useState<number | null>(null);

  // Inicializar DB y cargar clips existentes
  useEffect(() => {
    async function init() {
      try {
        await initDatabase();
        setDbReady(true);
        const existing = await invoke<Clip[]>("get_clips", { limit: 50, offset: 0 });
        setClips(existing);
      } catch (err) {
        setError(String(err));
      }
    }
    init();
  }, []);

  // Escuchar nuevos clips del monitor de clipboard
  useEffect(() => {
    const unlisten = listen<Clip>("new-clip", (event) => {
      setClips((prev) => [event.payload, ...prev]);
    });
    return () => { unlisten.then((fn) => fn()); };
  }, []);

  // Click en un clip: copiar al clipboard
  async function handleCopy(clip: Clip) {
    try {
      await invoke("copy_to_clipboard", { id: clip.id });
      // Feedback visual breve
      setCopiedId(clip.id);
      setTimeout(() => setCopiedId(null), 1000);
    } catch (err) {
      console.error("Copy failed:", err);
    }
  }

  if (error) {
    return <div className="p-4 text-red-500">Error: {error}</div>;
  }

  if (!dbReady) {
    return <div className="p-4 text-gray-400">Loading...</div>;
  }

  return (
    <main className="min-h-screen bg-gray-900 text-white p-4">
      <h1 className="text-xl font-bold mb-4">Pane — Clipboard Monitor</h1>
      <p className="text-gray-400 text-sm mb-4">
        {clips.length} clips — click to copy
      </p>
      <div className="space-y-2">
        {clips.map((clip) => (
          <div
            key={clip.id}
            onClick={() => handleCopy(clip)}
            className={`bg-gray-800 rounded-lg p-3 border cursor-pointer transition-colors ${
              copiedId === clip.id
                ? "border-green-500 bg-green-900/20"
                : "border-gray-700 hover:border-gray-500"
            }`}
          >
            <div className="flex justify-between items-start">
              {clip.content_type === "image" && clip.image_base64 ? (
                <img
                  src={`data:image/png;base64,${clip.image_base64}`}
                  alt={clip.preview || "Image"}
                  className="max-h-24 rounded"
                />
              ) : (
                <p className="text-sm text-gray-200 font-mono whitespace-pre-wrap break-all line-clamp-3">
                  {clip.preview || clip.content}
                </p>
              )}
              <div className="flex items-center gap-2 ml-2 shrink-0">
                {copiedId === clip.id && (
                  <span className="text-xs text-green-400">Copied!</span>
                )}
                <span className="text-xs text-gray-500">{clip.content_type}</span>
              </div>
            </div>
            <p className="text-xs text-gray-500 mt-1">{clip.created_at}</p>
          </div>
        ))}
      </div>
    </main>
  );
}

export default App;
