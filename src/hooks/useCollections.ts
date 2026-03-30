import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { Collection } from "../types";

export function useCollections() {
  const [collections, setCollections] = useState<Collection[]>([]);

  useEffect(() => {
    invoke<Collection[]>("get_collections").then(setCollections).catch(console.error);
  }, []);

  const createCollection = useCallback(async (name: string, icon?: string) => {
    const col = await invoke<Collection>("create_collection", { name, icon });
    setCollections((prev) => [...prev, col].sort((a, b) => a.name.localeCompare(b.name)));
    return col;
  }, []);

  const deleteCollection = useCallback(async (id: number) => {
    await invoke("delete_collection", { id });
    setCollections((prev) => prev.filter((c) => c.id !== id));
  }, []);

  const renameCollection = useCallback(async (id: number, name: string) => {
    await invoke("rename_collection", { id, name });
    setCollections((prev) =>
      prev.map((c) => (c.id === id ? { ...c, name } : c))
        .sort((a, b) => a.name.localeCompare(b.name))
    );
  }, []);

  const setClipCollection = useCallback(async (clipId: number, collectionId: number | null) => {
    await invoke("set_clip_collection", { clipId, collectionId });
  }, []);

  return { collections, createCollection, deleteCollection, renameCollection, setClipCollection };
}
