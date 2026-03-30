import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { Clip, FilterType } from "../types";
import { CLIPS_PAGE_SIZE } from "../lib/constants";

export function useClips() {
  const [clips, setClips] = useState<Clip[]>([]);
  const [loading, setLoading] = useState(true);
  const [filter, setFilter] = useState<FilterType>("all");
  const [hasMore, setHasMore] = useState(true);
  const offsetRef = useRef(0);

  // Cargar clips iniciales
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

  // Escuchar nuevos clips en tiempo real
  useEffect(() => {
    const unlisten = listen<Clip>("new-clip", (event) => {
      setClips((prev) => [event.payload, ...prev]);
      offsetRef.current += 1;
    });
    return () => { unlisten.then((fn) => fn()); };
  }, []);

  // Cargar más clips (scroll infinito)
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

  // Borrar un clip
  const deleteClip = useCallback(async (id: number) => {
    await invoke("delete_clip", { id });
    setClips((prev) => prev.filter((c) => c.id !== id));
  }, []);

  // Toggle pin
  const togglePin = useCallback(async (id: number) => {
    const newVal = await invoke<boolean>("toggle_pin", { id });
    setClips((prev) =>
      prev.map((c) => (c.id === id ? { ...c, is_pinned: newVal } : c))
    );
  }, []);

  // Toggle favorite
  const toggleFavorite = useCallback(async (id: number) => {
    const newVal = await invoke<boolean>("toggle_favorite", { id });
    setClips((prev) =>
      prev.map((c) => (c.id === id ? { ...c, is_favorite: newVal } : c))
    );
  }, []);

  // Limpiar historial
  const clearHistory = useCallback(async () => {
    await invoke("clear_history");
    setClips((prev) => prev.filter((c) => c.is_pinned || c.is_favorite));
  }, []);

  // Filtrar clips según el filtro activo
  const filteredClips = clips.filter((clip) => {
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
    setFilter,
    hasMore,
    loadMore,
    deleteClip,
    togglePin,
    toggleFavorite,
    clearHistory,
  };
}
