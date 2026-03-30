import { useEffect, useState } from "react";
import { initDatabase } from "./lib/db";
import { useClips } from "./hooks/useClips";
import { Sidebar } from "./components/Sidebar";
import { ClipList } from "./components/ClipList";
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
    return (
      <div className="h-screen bg-gray-950 flex items-center justify-center p-4 text-red-500">
        Error: {error}
      </div>
    );
  }

  if (!dbReady) {
    return (
      <div className="h-screen bg-gray-950 flex items-center justify-center text-gray-400">
        Loading...
      </div>
    );
  }

  return <MainView />;
}

function MainView() {
  const {
    clips,
    allClips,
    filter,
    setFilter,
    hasMore,
    loadMore,
    deleteClip,
    togglePin,
    toggleFavorite,
  } = useClips();

  // Contar clips por categoría para el sidebar
  const clipCounts: Record<string, number> = {
    all: allClips.length,
    pinned: allClips.filter((c) => c.is_pinned).length,
    favorites: allClips.filter((c) => c.is_favorite).length,
    text: allClips.filter((c) => c.content_type === "text").length,
    image: allClips.filter((c) => c.content_type === "image").length,
    url: allClips.filter((c) => c.content_type === "url").length,
    code: allClips.filter((c) => c.content_type === "code").length,
    color: allClips.filter((c) => c.content_type === "color").length,
  };

  return (
    <div className="h-screen flex bg-gray-950 text-white overflow-hidden">
      <Sidebar
        filter={filter}
        onFilterChange={setFilter}
        clipCounts={clipCounts}
      />
      <main className="flex-1 flex flex-col min-w-0">
        {/* Header */}
        <div className="px-4 py-3 border-b border-gray-800 flex items-center justify-between">
          <div>
            <h2 className="text-sm font-semibold text-gray-200">
              {filter === "all" ? "All Clips" : filter.charAt(0).toUpperCase() + filter.slice(1)}
            </h2>
            <p className="text-xs text-gray-500">{clips.length} clips</p>
          </div>
        </div>
        {/* Clip list */}
        <ClipList
          clips={clips}
          hasMore={hasMore}
          onLoadMore={loadMore}
          onDelete={deleteClip}
          onTogglePin={togglePin}
          onToggleFavorite={toggleFavorite}
        />
      </main>
    </div>
  );
}

export default App;
