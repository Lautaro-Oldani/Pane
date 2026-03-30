import { useEffect, useState } from "react";
import { initDatabase } from "./lib/db";
import "./App.css";

function App() {
  const [dbReady, setDbReady] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    initDatabase()
      .then(() => setDbReady(true))
      .catch((err) => setError(String(err)));
  }, []);

  if (error) {
    return <div className="p-4 text-red-500">Error initializing DB: {error}</div>;
  }

  if (!dbReady) {
    return <div className="p-4 text-gray-400">Loading...</div>;
  }

  return (
    <main className="p-4">
      <h1 className="text-2xl font-bold text-white">Pane</h1>
      <p className="text-gray-400 mt-2">Database ready. Clipboard manager coming soon.</p>
    </main>
  );
}

export default App;
