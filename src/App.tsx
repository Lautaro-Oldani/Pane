import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { initDatabase } from "./lib/db";
import { useClips } from "./hooks/useClips";
import { useSearch } from "./hooks/useSearch";
import { Sidebar } from "./components/Sidebar";
import { SearchBar } from "./components/SearchBar";
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

  // Búsqueda fuzzy sobre los clips ya filtrados
  const { query, setQuery, results } = useSearch(clips);

  // Keyboard navigation
  const [selectedIndex, setSelectedIndex] = useState(-1);

  // Reset selection cuando cambian los resultados
  useEffect(() => {
    setSelectedIndex(-1);
  }, [query, filter]);

  const handleKeyDown = useCallback(
    async (e: KeyboardEvent) => {
      switch (e.key) {
        case "ArrowDown":
          e.preventDefault();
          setSelectedIndex((prev) => Math.min(prev + 1, results.length - 1));
          break;
        case "ArrowUp":
          e.preventDefault();
          setSelectedIndex((prev) => Math.max(prev - 1, -1));
          break;
        case "Enter":
          e.preventDefault();
          if (selectedIndex >= 0 && selectedIndex < results.length) {
            const clip = results[selectedIndex];
            await invoke("copy_to_clipboard", { id: clip.id });
            getCurrentWindow().hide();
          }
          break;
        case "Escape":
          e.preventDefault();
          getCurrentWindow().hide();
          break;
      }
    },
    [results, selectedIndex]
  );

  useEffect(() => {
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [handleKeyDown]);

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
        {/* Header con search */}
        <div className="px-4 py-3 border-b border-gray-800 space-y-2">
          <SearchBar value={query} onChange={setQuery} />
          <div className="flex items-center justify-between">
            <h2 className="text-sm font-semibold text-gray-200">
              {filter === "all" ? "All Clips" : filter.charAt(0).toUpperCase() + filter.slice(1)}
            </h2>
            <p className="text-xs text-gray-500">
              {query ? `${results.length} results` : `${clips.length} clips`}
            </p>
          </div>
        </div>
        {/* Clip list — muestra resultados de búsqueda si hay query */}
        <ClipList
          clips={results}
          selectedIndex={selectedIndex}
          hasMore={!query && hasMore}
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
