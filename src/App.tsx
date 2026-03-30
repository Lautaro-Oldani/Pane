import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { initDatabase } from "./lib/db";
import { useClips } from "./hooks/useClips";
import { useSearch } from "./hooks/useSearch";
import { useCollections } from "./hooks/useCollections";
import { useSettings } from "./hooks/useSettings";
import { Sidebar } from "./components/Sidebar";
import { SearchBar } from "./components/SearchBar";
import { ClipList } from "./components/ClipList";
import { Settings } from "./components/Settings";
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
    selectedCollectionId,
    changeFilter,
    hasMore,
    loadMore,
    deleteClip,
    togglePin,
    toggleFavorite,
    updateClipCollection,
  } = useClips();

  const { collections, createCollection, deleteCollection, setClipCollection } = useCollections();
  const { settings, updateSetting } = useSettings();
  const { query, setQuery, results } = useSearch(clips);
  const [showSettings, setShowSettings] = useState(false);

  // Keyboard navigation
  const [selectedIndex, setSelectedIndex] = useState(-1);

  useEffect(() => {
    setSelectedIndex(-1);
  }, [query, filter, selectedCollectionId]);

  const handleMoveToCollection = useCallback(async (clipId: number, collectionId: number | null) => {
    await setClipCollection(clipId, collectionId);
    updateClipCollection(clipId, collectionId);
  }, [setClipCollection, updateClipCollection]);

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
            invoke("hide_app_window");
          }
          break;
        case "Escape":
          e.preventDefault();
          invoke("hide_app_window");
          break;
      }
    },
    [results, selectedIndex]
  );

  useEffect(() => {
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [handleKeyDown]);

  // Contar clips por categoría
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

  // Contar clips por colección
  const collectionClipCounts: Record<number, number> = {};
  for (const col of collections) {
    collectionClipCounts[col.id] = allClips.filter((c) => c.collection_id === col.id).length;
  }

  // Título del header
  const headerTitle = selectedCollectionId !== null
    ? collections.find((c) => c.id === selectedCollectionId)?.name || "Collection"
    : filter === "all" ? "All Clips" : filter.charAt(0).toUpperCase() + filter.slice(1);

  return (
    <div className="h-screen flex bg-gray-950 text-white overflow-hidden">
      <Sidebar
        filter={filter}
        selectedCollectionId={selectedCollectionId}
        onFilterChange={changeFilter}
        clipCounts={clipCounts}
        collections={collections}
        collectionClipCounts={collectionClipCounts}
        onCreateCollection={createCollection}
        onDeleteCollection={deleteCollection}
        onOpenSettings={() => setShowSettings(true)}
      />
      <main className="flex-1 flex flex-col min-w-0">
        {showSettings ? (
          <Settings
            settings={settings}
            onUpdate={updateSetting}
            onClose={() => setShowSettings(false)}
          />
        ) : (
          <>
            <div className="px-4 py-3 border-b border-gray-800 space-y-2">
              <SearchBar value={query} onChange={setQuery} />
              <div className="flex items-center justify-between">
                <h2 className="text-sm font-semibold text-gray-200">{headerTitle}</h2>
                <p className="text-xs text-gray-500">
                  {query ? `${results.length} results` : `${clips.length} clips`}
                </p>
              </div>
            </div>
            <ClipList
              clips={results}
              selectedIndex={selectedIndex}
              hasMore={!query && hasMore}
              collections={collections}
              onLoadMore={loadMore}
              onDelete={deleteClip}
              onTogglePin={togglePin}
              onToggleFavorite={toggleFavorite}
              onMoveToCollection={handleMoveToCollection}
            />
          </>
        )}
      </main>
    </div>
  );
}

export default App;
