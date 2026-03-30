import type { FilterType } from "../types";

interface SidebarProps {
  filter: FilterType;
  onFilterChange: (filter: FilterType) => void;
  clipCounts: Record<string, number>;
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

export function Sidebar({ filter, onFilterChange, clipCounts }: SidebarProps) {
  return (
    <aside className="w-48 bg-gray-950 border-r border-gray-800 flex flex-col h-full">
      <div className="p-3 border-b border-gray-800">
        <h1 className="text-lg font-bold text-white tracking-tight">Pane</h1>
      </div>

      <nav className="flex-1 overflow-y-auto p-2 space-y-1">
        {NAV_ITEMS.map((item) => (
          <NavButton
            key={item.key}
            icon={item.icon}
            label={item.label}
            count={clipCounts[item.key]}
            active={filter === item.key}
            onClick={() => onFilterChange(item.key)}
          />
        ))}

        <div className="border-t border-gray-800 my-2" />
        <p className="text-xs text-gray-500 px-2 py-1 uppercase tracking-wider">
          Types
        </p>

        {TYPE_ITEMS.map((item) => (
          <NavButton
            key={item.key}
            icon={item.icon}
            label={item.label}
            count={clipCounts[item.key]}
            active={filter === item.key}
            onClick={() => onFilterChange(item.key)}
          />
        ))}
      </nav>
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
      <span className="flex-1 text-left">{label}</span>
      {count !== undefined && count > 0 && (
        <span className="text-xs text-gray-500">{count}</span>
      )}
    </button>
  );
}
