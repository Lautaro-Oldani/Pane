import { useEffect, useRef } from "react";
import type { Clip } from "../types";
import { ClipItem } from "./ClipItem";

interface ClipListProps {
  clips: Clip[];
  selectedIndex: number;
  hasMore: boolean;
  collections?: { id: number; name: string; icon: string | null }[];
  onLoadMore: () => void;
  onDelete: (id: number) => void;
  onTogglePin: (id: number) => void;
  onToggleFavorite: (id: number) => void;
  onMoveToCollection?: (clipId: number, collectionId: number | null) => void;
}

export function ClipList({
  clips,
  selectedIndex,
  hasMore,
  collections,
  onLoadMore,
  onDelete,
  onTogglePin,
  onToggleFavorite,
  onMoveToCollection,
}: ClipListProps) {
  const sentinelRef = useRef<HTMLDivElement>(null);
  const listRef = useRef<HTMLDivElement>(null);

  // Intersection Observer para scroll infinito
  useEffect(() => {
    if (!sentinelRef.current || !hasMore) return;
    const observer = new IntersectionObserver(
      (entries) => {
        if (entries[0].isIntersecting) onLoadMore();
      },
      { threshold: 0.1 }
    );
    observer.observe(sentinelRef.current);
    return () => observer.disconnect();
  }, [hasMore, onLoadMore]);

  // Auto-scroll para mantener el item seleccionado visible
  useEffect(() => {
    if (selectedIndex < 0 || !listRef.current) return;
    const items = listRef.current.querySelectorAll("[data-clip-item]");
    const selected = items[selectedIndex] as HTMLElement | undefined;
    selected?.scrollIntoView({ block: "nearest", behavior: "smooth" });
  }, [selectedIndex]);

  if (clips.length === 0) {
    return (
      <div className="flex-1 flex items-center justify-center text-gray-500">
        <div className="text-center space-y-2">
          <p className="text-4xl">📋</p>
          <p className="text-sm font-medium">No clips yet</p>
          <p className="text-xs">Copy something anywhere to see it here</p>
          <div className="mt-4 pt-4 border-t border-gray-800">
            <p className="text-xs">
              <kbd className="px-1.5 py-0.5 bg-gray-800 rounded text-gray-400 text-[11px]">Ctrl+Shift+V</kbd>
              <span className="ml-1.5">to open Pane from anywhere</span>
            </p>
            <p className="text-xs mt-1">
              <kbd className="px-1.5 py-0.5 bg-gray-800 rounded text-gray-400 text-[11px]">Esc</kbd>
              <span className="ml-1.5">to dismiss</span>
            </p>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div ref={listRef} className="flex-1 overflow-y-auto p-3 space-y-2">
      {clips.map((clip, index) => (
        <ClipItem
          key={clip.id}
          clip={clip}
          selected={index === selectedIndex}
          collections={collections}
          onDelete={onDelete}
          onTogglePin={onTogglePin}
          onToggleFavorite={onToggleFavorite}
          onMoveToCollection={onMoveToCollection}
        />
      ))}
      {hasMore && (
        <div ref={sentinelRef} className="h-8 flex items-center justify-center">
          <span className="text-xs text-gray-600">Loading more...</span>
        </div>
      )}
    </div>
  );
}
