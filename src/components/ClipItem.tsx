import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { Clip } from "../types";
import { timeAgo } from "../lib/time";

// Colores de badge según el tipo de contenido
const TYPE_BADGE: Record<string, { label: string; color: string }> = {
  text: { label: "Text", color: "bg-gray-600" },
  image: { label: "Image", color: "bg-purple-600" },
  url: { label: "Link", color: "bg-blue-600" },
  code: { label: "Code", color: "bg-green-600" },
  color: { label: "Color", color: "bg-pink-600" },
  html: { label: "HTML", color: "bg-orange-600" },
};

interface ClipItemProps {
  clip: Clip;
  selected?: boolean;
  onDelete: (id: number) => void;
  onTogglePin: (id: number) => void;
  onToggleFavorite: (id: number) => void;
}

export function ClipItem({ clip, selected, onDelete, onTogglePin, onToggleFavorite }: ClipItemProps) {
  const [copied, setCopied] = useState(false);
  const [showActions, setShowActions] = useState(false);
  const badge = TYPE_BADGE[clip.content_type] || TYPE_BADGE.text;

  async function handleCopy() {
    try {
      await invoke("copy_to_clipboard", { id: clip.id });
      setCopied(true);
      setTimeout(() => setCopied(false), 1000);
    } catch (err) {
      console.error("Copy failed:", err);
    }
  }

  return (
    <div
      data-clip-item
      onClick={handleCopy}
      onMouseEnter={() => setShowActions(true)}
      onMouseLeave={() => setShowActions(false)}
      className={`group relative rounded-lg p-3 border cursor-pointer transition-all ${
        copied
          ? "border-green-500/50 bg-green-500/5"
          : selected
            ? "border-blue-500/50 bg-blue-500/10"
            : "border-gray-800 bg-gray-900 hover:border-gray-600 hover:bg-gray-800/50"
      }`}
    >
      {/* Header: badge + timestamp */}
      <div className="flex items-center justify-between mb-1.5">
        <div className="flex items-center gap-1.5">
          <span className={`text-[10px] px-1.5 py-0.5 rounded font-medium text-white ${badge.color}`}>
            {badge.label}
          </span>
          {clip.is_pinned && <span className="text-[10px]">📌</span>}
          {clip.is_favorite && <span className="text-[10px]">⭐</span>}
        </div>
        <div className="flex items-center gap-2">
          {copied && <span className="text-xs text-green-400 font-medium">Copied!</span>}
          <span className="text-[11px] text-gray-500">{timeAgo(clip.created_at)}</span>
        </div>
      </div>

      {/* Content */}
      {clip.content_type === "image" && clip.image_base64 ? (
        <img
          src={`data:image/png;base64,${clip.image_base64}`}
          alt={clip.preview || "Image"}
          className="max-h-32 rounded border border-gray-700"
          loading="lazy"
        />
      ) : (
        <p className="text-sm text-gray-300 font-mono whitespace-pre-wrap break-all line-clamp-4 leading-relaxed">
          {clip.preview || clip.content}
        </p>
      )}

      {/* Action buttons (visible on hover) */}
      {showActions && (
        <div className="absolute top-2 right-2 flex gap-1" onClick={(e) => e.stopPropagation()}>
          <ActionBtn
            title={clip.is_pinned ? "Unpin" : "Pin"}
            onClick={() => onTogglePin(clip.id)}
          >
            📌
          </ActionBtn>
          <ActionBtn
            title={clip.is_favorite ? "Unfavorite" : "Favorite"}
            onClick={() => onToggleFavorite(clip.id)}
          >
            ⭐
          </ActionBtn>
          <ActionBtn title="Delete" onClick={() => onDelete(clip.id)}>
            🗑️
          </ActionBtn>
        </div>
      )}
    </div>
  );
}

function ActionBtn({
  children,
  title,
  onClick,
}: {
  children: React.ReactNode;
  title: string;
  onClick: () => void;
}) {
  return (
    <button
      title={title}
      onClick={onClick}
      className="w-7 h-7 flex items-center justify-center rounded bg-gray-800 hover:bg-gray-700 border border-gray-700 text-xs transition-colors"
    >
      {children}
    </button>
  );
}
