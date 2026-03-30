import { useState } from "react";
import type { Collection, FilterType } from "../types";

interface SidebarProps {
  filter: FilterType;
  selectedCollectionId: number | null;
  onFilterChange: (filter: FilterType, collectionId?: number) => void;
  clipCounts: Record<string, number>;
  collections: Collection[];
  collectionClipCounts: Record<number, number>;
  onCreateCollection: (name: string) => void;
  onDeleteCollection: (id: number) => void;
  onOpenSettings: () => void;
  onOpenSupport: () => void;
}

const NAV_ITEMS: { key: FilterType; label: string; icon: string }[] = [
  { key: "all", label: "All Clips", icon: "📋" },
  { key: "pinned", label: "Pinned", icon: "📌" },
  { key: "favorites", label: "Favorites", icon: "⭐" },
];

const TYPE_ITEMS: { key: FilterType; label: string; icon: string }[] = [
  { key: "text", label: "Text", icon: "📝" },
  { key: "image", label: "Images", icon: "🖼️" },
  { key: "url", label: "Links", icon: "🔗" },
  { key: "code", label: "Code", icon: "💻" },
  { key: "color", label: "Colors", icon: "🎨" },
];

export function Sidebar({
  filter,
  selectedCollectionId,
  onFilterChange,
  clipCounts,
  collections,
  collectionClipCounts,
  onCreateCollection,
  onDeleteCollection,
  onOpenSettings,
  onOpenSupport,
}: SidebarProps) {
  const [showNewInput, setShowNewInput] = useState(false);
  const [newName, setNewName] = useState("");

  function handleCreate() {
    const name = newName.trim();
    if (!name) return;
    onCreateCollection(name);
    setNewName("");
    setShowNewInput(false);
  }

  return (
    <aside className="w-48 theme-bg-secondary border-r theme-border flex flex-col h-full">
      <div className="p-3 border-b theme-border">
        <h1 className="text-lg font-bold theme-text tracking-tight">Pane</h1>
      </div>

      <nav className="flex-1 overflow-y-auto p-2 space-y-1">
        {NAV_ITEMS.map((item) => (
          <NavButton
            key={item.key}
            icon={item.icon}
            label={item.label}
            count={clipCounts[item.key]}
            active={filter === item.key && selectedCollectionId === null}
            onClick={() => onFilterChange(item.key)}
          />
        ))}

        <div className="border-t theme-border my-2" />
        <p className="text-xs text-gray-500 px-2 py-1 uppercase tracking-wider">Types</p>
        {TYPE_ITEMS.map((item) => (
          <NavButton
            key={item.key}
            icon={item.icon}
            label={item.label}
            count={clipCounts[item.key]}
            active={filter === item.key && selectedCollectionId === null}
            onClick={() => onFilterChange(item.key)}
          />
        ))}

        {/* Collections */}
        <div className="border-t theme-border my-2" />
        <div className="flex items-center justify-between px-2 py-1">
          <p className="text-xs text-gray-500 uppercase tracking-wider">Collections</p>
          <button
            onClick={() => setShowNewInput(true)}
            className="text-gray-500 hover:text-gray-300 text-sm leading-none"
            title="New collection"
          >
            +
          </button>
        </div>

        {showNewInput && (
          <div className="px-1">
            <input
              autoFocus
              value={newName}
              onChange={(e) => setNewName(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter") handleCreate();
                if (e.key === "Escape") { setShowNewInput(false); setNewName(""); }
              }}
              onBlur={() => { if (!newName.trim()) setShowNewInput(false); }}
              placeholder="Name..."
              className="w-full bg-gray-800 border border-gray-700 rounded px-2 py-1 text-sm text-gray-200 placeholder-gray-500 focus:outline-none focus:border-gray-500"
            />
          </div>
        )}

        {collections.map((col) => (
          <div key={col.id} className="group flex items-center">
            <NavButton
              icon={col.icon || "📁"}
              label={col.name}
              count={collectionClipCounts[col.id]}
              active={selectedCollectionId === col.id}
              onClick={() => onFilterChange("collection" as FilterType, col.id)}
            />
            <button
              onClick={() => onDeleteCollection(col.id)}
              className="opacity-0 group-hover:opacity-100 text-gray-600 hover:text-red-400 text-xs px-1 shrink-0"
              title="Delete collection"
            >
              ✕
            </button>
          </div>
        ))}
      </nav>

      <div className="p-2 border-t theme-border space-y-1">
        <button
          onClick={onOpenSupport}
          className="w-full flex items-center gap-2 px-2 py-1.5 rounded-md text-sm text-[#FF5E5B] hover:bg-red-500/10 transition-colors"
        >
          <span className="text-base">☕</span>
          <span>Support Pane</span>
        </button>
        <button
          onClick={onOpenSettings}
          className="w-full flex items-center gap-2 px-2 py-1.5 rounded-md text-sm text-gray-400 hover:text-gray-200 hover:bg-gray-800/50 transition-colors"
        >
          <span className="text-base">⚙️</span>
          <span>Settings</span>
        </button>
      </div>
    </aside>
  );
}

function NavButton({
  icon,
  label,
  count,
  active,
  onClick,
}: {
  icon: string;
  label: string;
  count?: number;
  active: boolean;
  onClick: () => void;
}) {
  return (
    <button
      onClick={onClick}
      className={`w-full flex items-center gap-2 px-2 py-1.5 rounded-md text-sm transition-colors ${
        active
          ? "bg-gray-800 text-white"
          : "text-gray-400 hover:text-gray-200 hover:bg-gray-800/50"
      }`}
    >
      <span className="text-base">{icon}</span>
      <span className="flex-1 text-left truncate">{label}</span>
      {count !== undefined && count > 0 && (
        <span className="text-xs text-gray-500">{count}</span>
      )}
    </button>
  );
}
