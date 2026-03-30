import { useMemo, useState } from "react";
import Fuse from "fuse.js";
import type { Clip } from "../types";

// Configuración de fuse.js para búsqueda fuzzy
const FUSE_OPTIONS = {
  keys: [
    { name: "content", weight: 1.0 },   // El contenido tiene más peso
    { name: "preview", weight: 0.5 },    // El preview menos
  ],
  threshold: 0.4,      // 0 = match exacto, 1 = match cualquier cosa
  includeScore: true,
  minMatchCharLength: 2,
};

export function useSearch(clips: Clip[]) {
  const [query, setQuery] = useState("");

  // Crear el índice de Fuse (se recrea cuando cambian los clips)
  const fuse = useMemo(() => new Fuse(clips, FUSE_OPTIONS), [clips]);

  // Filtrar clips con búsqueda fuzzy
  const results = useMemo(() => {
    if (!query.trim()) return clips;
    return fuse.search(query).map((result) => result.item);
  }, [fuse, query, clips]);

  return { query, setQuery, results };
}
