import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { Clip, FilterType } from "../types";
import { CLIPS_PAGE_SIZE } from "../lib/constants";

export function useClips() {
  const [clips, setClips] = useState<Clip[]>([]);
  const [loading, setLoading] = useState(true);
  const [filter, setFilter] = useState<FilterType>("all");
  const [selectedCollectionId, setSelectedCollectionId] = useState<number | null>(null);
  const [hasMore, setHasMore] = useState(true);
  const offsetRef = useRef(0);

  useEffect(() => {
    async function load() {
      try {
        const data = await invoke<Clip[]>("get_clips", {
          limit: CLIPS_PAGE_SIZE,
          offset: 0,
        });
        setClips(data);
        setHasMore(data.length === CLIPS_PAGE_SIZE);
        offsetRef.current = data.length;
      } catch (err) {
        console.error("Failed to load clips:", err);
      } finally {
        setLoading(false);
      }
    }
    load();
  }, []);

  useEffect(() => {
    const unlisten = listen<Clip>("new-clip", (event) => {
      setClips((prev) => [event.payload, ...prev]);
      offsetRef.current += 1;
    });
    return () => { unlisten.then((fn) => fn()); };
  }, []);

  const loadMore = useCallback(async () => {
    if (!hasMore) return;
    try {
      const data = await invoke<Clip[]>("get_clips", {
        limit: CLIPS_PAGE_SIZE,
        offset: offsetRef.current,
      });
      setClips((prev) => [...prev, ...data]);
      setHasMore(data.length === CLIPS_PAGE_SIZE);
      offsetRef.current += data.length;
    } catch (err) {
      console.error("Failed to load more clips:", err);
    }
  }, [hasMore]);

  const deleteClip = useCallback(async (id: number) => {
    await invoke("delete_clip", { id });
    setClips((prev) => prev.filter((c) => c.id !== id));
  }, []);

  const togglePin = useCallback(async (id: number) => {
    const newVal = await invoke<boolean>("toggle_pin", { id });
    setClips((prev) =>
      prev.map((c) => (c.id === id ? { ...c, is_pinned: newVal } : c))
    );
  }, []);

  const toggleFavorite = useCallback(async (id: number) => {
    const newVal = await invoke<boolean>("toggle_favorite", { id });
    setClips((prev) =>
      prev.map((c) => (c.id === id ? { ...c, is_favorite: newVal } : c))
    );
  }, []);

  const clearHistory = useCallback(async () => {
    await invoke("clear_history");
    setClips((prev) => prev.filter((c) => c.is_pinned || c.is_favorite));
  }, []);

  // Actualizar collection_id de un clip en el estado local
  const updateClipCollection = useCallback((clipId: number, collectionId: number | null) => {
    setClips((prev) =>
      prev.map((c) => (c.id === clipId ? { ...c, collection_id: collectionId } : c))
    );
  }, []);

  // Cambiar filtro (para colecciones se usa setFilter + setSelectedCollectionId)
  const changeFilter = useCallback((newFilter: FilterType, collectionId?: number) => {
    if (collectionId !== undefined) {
      setFilter("all"); // reset type filter
      setSelectedCollectionId(collectionId);
    } else {
      setFilter(newFilter);
      setSelectedCollectionId(null);
    }
  }, []);

  // Filtrar clips
  const filteredClips = clips.filter((clip) => {
    // Si hay colección seleccionada, filtrar por colección
    if (selectedCollectionId !== null) {
      return clip.collection_id === selectedCollectionId;
    }
    switch (filter) {
      case "pinned": return clip.is_pinned;
      case "favorites": return clip.is_favorite;
      case "all": return true;
      default: return clip.content_type === filter;
    }
  });

  return {
    clips: filteredClips,
    allClips: clips,
    loading,
    filter,
    selectedCollectionId,
    changeFilter,
    hasMore,
    loadMore,
    deleteClip,
    togglePin,
    toggleFavorite,
    clearHistory,
    updateClipCollection,
  };
}
