import { useEffect, useRef } from "react";

interface SearchBarProps {
  value: string;
  onChange: (value: string) => void;
}

export function SearchBar({ value, onChange }: SearchBarProps) {
  const inputRef = useRef<HTMLInputElement>(null);

  // Escuchar evento del backend para dar foco al search al abrir la ventana
  useEffect(() => {
    // Cuando la ventana se hace visible, focus al input
    const handleFocus = () => inputRef.current?.focus();
    window.addEventListener("focus", handleFocus);
    return () => window.removeEventListener("focus", handleFocus);
  }, []);

  return (
    <div className="relative">
      <input
        ref={inputRef}
        type="text"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        placeholder="Search clips..."
        className="w-full bg-gray-800 border border-gray-700 rounded-lg px-3 py-2 pl-9
                   text-sm text-gray-200 placeholder-gray-500
                   focus:outline-none focus:border-gray-500 focus:ring-1 focus:ring-gray-500
                   transition-colors"
      />
      <svg
        className="absolute left-2.5 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-500"
        fill="none"
        stroke="currentColor"
        viewBox="0 0 24 24"
      >
        <path
          strokeLinecap="round"
          strokeLinejoin="round"
          strokeWidth={2}
          d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
        />
      </svg>
      {value && (
        <button
          onClick={() => onChange("")}
          className="absolute right-2.5 top-1/2 -translate-y-1/2 text-gray-500 hover:text-gray-300 text-xs"
        >
          ✕
        </button>
      )}
    </div>
  );
}
