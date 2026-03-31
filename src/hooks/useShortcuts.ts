import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

export interface Shortcut {
  id: number;
  trigger: string;
  content: string;
  created_at: string;
}

export function useShortcuts() {
  const [shortcuts, setShortcuts] = useState<Shortcut[]>([]);

  useEffect(() => {
    invoke<Shortcut[]>("get_shortcuts").then(setShortcuts).catch(console.error);
  }, []);

  const createShortcut = useCallback(async (trigger: string, content: string) => {
    const shortcut = await invoke<Shortcut>("create_shortcut", { trigger, content });
    setShortcuts((prev) => [...prev, shortcut].sort((a, b) => a.trigger.localeCompare(b.trigger)));
    return shortcut;
  }, []);

  const deleteShortcut = useCallback(async (id: number) => {
    await invoke("delete_shortcut", { id });
    setShortcuts((prev) => prev.filter((s) => s.id !== id));
  }, []);

  const updateShortcut = useCallback(async (id: number, trigger: string, content: string) => {
    await invoke("update_shortcut", { id, trigger, content });
    setShortcuts((prev) =>
      prev.map((s) => (s.id === id ? { ...s, trigger, content } : s))
        .sort((a, b) => a.trigger.localeCompare(b.trigger))
    );
  }, []);

  return { shortcuts, createShortcut, deleteShortcut, updateShortcut };
}
