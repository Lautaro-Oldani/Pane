import { useEffect, useRef } from "react";
import type { Clip } from "../types";
import { ClipItem } from "./ClipItem";

interface ClipListProps {
  clips: Clip[];
  hasMore: boolean;
  onLoadMore: () => void;
  onDelete: (id: number) => void;
  onTogglePin: (id: number) => void;
  onToggleFavorite: (id: number) => void;
}

export function ClipList({
  clips,
  hasMore,
  onLoadMore,
  onDelete,
  onTogglePin,
  onToggleFavorite,
}: ClipListProps) {
  const sentinelRef = useRef<HTMLDivElement>(null);

  // Intersection Observer para scroll infinito:
  // cuando el "sentinel" (div invisible al final) entra en el viewport,
  // cargamos más clips automáticamente.
  useEffect(() => {
    if (!sentinelRef.current || !hasMore) return;

    const observer = new IntersectionObserver(
      (entries) => {
        if (entries[0].isIntersecting) {
          onLoadMore();
        }
      },
      { threshold: 0.1 }
    );

    observer.observe(sentinelRef.current);
    return () => observer.disconnect();
  }, [hasMore, onLoadMore]);

  if (clips.length === 0) {
    return (
      <div className="flex-1 flex items-center justify-center text-gray-500">
        <div className="text-center">
          <p className="text-3xl mb-2">📋</p>
          <p className="text-sm">No clips yet</p>
          <p className="text-xs mt-1">Copy something to get started</p>
        </div>
      </div>
    );
  }

  return (
    <div className="flex-1 overflow-y-auto p-3 space-y-2">
      {clips.map((clip) => (
        <ClipItem
          key={clip.id}
          clip={clip}
          onDelete={onDelete}
          onTogglePin={onTogglePin}
          onToggleFavorite={onToggleFavorite}
        />
      ))}

      {/* Sentinel para scroll infinito */}
      {hasMore && (
        <div ref={sentinelRef} className="h-8 flex items-center justify-center">
          <span className="text-xs text-gray-600">Loading more...</span>
        </div>
      )}
    </div>
  );
}
