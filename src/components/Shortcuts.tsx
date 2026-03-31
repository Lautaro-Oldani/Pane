import { useState } from "react";
import type { Shortcut } from "../hooks/useShortcuts";

interface ShortcutsProps {
  shortcuts: Shortcut[];
  onCreate: (trigger: string, content: string) => void;
  onDelete: (id: number) => void;
  onUpdate: (id: number, trigger: string, content: string) => void;
  onClose: () => void;
}

export function Shortcuts({ shortcuts, onCreate, onDelete, onUpdate, onClose }: ShortcutsProps) {
  const [showForm, setShowForm] = useState(false);
  const [editingId, setEditingId] = useState<number | null>(null);
  const [trigger, setTrigger] = useState("");
  const [content, setContent] = useState("");

  function handleSave() {
    let t = trigger.trim();
    const c = content.trim();
    if (!t || !c) return;

    // Forzar el prefijo /
    if (!t.startsWith("/")) {
      t = "/" + t;
    }

    if (editingId !== null) {
      onUpdate(editingId, t, c);
    } else {
      onCreate(t, c);
    }
    resetForm();
  }

  function handleEdit(shortcut: Shortcut) {
    setEditingId(shortcut.id);
    setTrigger(shortcut.trigger);
    setContent(shortcut.content);
    setShowForm(true);
  }

  function resetForm() {
    setShowForm(false);
    setEditingId(null);
    setTrigger("");
    setContent("");
  }

  return (
    <div className="flex-1 overflow-y-auto p-6">
      <div className="flex items-center justify-between mb-4">
        <h2 className="text-lg font-bold theme-text">Shortcuts</h2>
        <button
          onClick={onClose}
          className="text-gray-400 hover:text-white text-sm px-2 py-1 rounded hover:bg-gray-800"
        >
          ← Back
        </button>
      </div>

      <p className="text-xs text-gray-500 mb-4">
        Type a trigger (e.g. /greeting) anywhere and press space — it gets replaced by the content automatically.
      </p>

      {/* New shortcut button */}
      {!showForm && (
        <button
          onClick={() => setShowForm(true)}
          className="w-full py-2 px-4 rounded-lg border border-dashed border-gray-700 text-sm text-gray-400 hover:text-gray-200 hover:border-gray-500 transition-colors mb-4"
        >
          + New Shortcut
        </button>
      )}

      {/* Form */}
      {showForm && (
        <div className="bg-gray-900 border border-gray-700 rounded-lg p-4 mb-4 space-y-3">
          <div>
            <label className="text-xs text-gray-400 block mb-1">Trigger (auto-prefixed with /)</label>
            <input
              autoFocus
              value={trigger}
              onChange={(e) => setTrigger(e.target.value)}
              placeholder="greeting"
              className="w-full bg-gray-800 border border-gray-700 rounded px-3 py-2 text-sm text-gray-200 placeholder-gray-500 focus:outline-none focus:border-gray-500 font-mono"
            />
          </div>
          <div>
            <label className="text-xs text-gray-400 block mb-1">Content</label>
            <textarea
              value={content}
              onChange={(e) => setContent(e.target.value)}
              placeholder="Hello! Welcome to our service. How can I help you today?"
              rows={3}
              className="w-full bg-gray-800 border border-gray-700 rounded px-3 py-2 text-sm text-gray-200 placeholder-gray-500 focus:outline-none focus:border-gray-500 resize-none"
            />
          </div>
          <div className="flex gap-2 justify-end">
            <button
              onClick={resetForm}
              className="px-3 py-1.5 text-sm text-gray-400 hover:text-gray-200 transition-colors"
            >
              Cancel
            </button>
            <button
              onClick={handleSave}
              disabled={!trigger.trim() || !content.trim()}
              className="px-4 py-1.5 bg-blue-600 hover:bg-blue-500 text-white text-sm rounded transition-colors disabled:opacity-40"
            >
              {editingId !== null ? "Update" : "Create"}
            </button>
          </div>
        </div>
      )}

      {/* List */}
      <div className="space-y-2">
        {shortcuts.map((shortcut) => (
          <div
            key={shortcut.id}
            className="bg-gray-900 border border-gray-800 rounded-lg p-3 group hover:border-gray-600 transition-colors"
          >
            <div className="flex items-start justify-between">
              <div className="min-w-0 flex-1">
                <p className="text-sm font-mono text-blue-400">{shortcut.trigger}</p>
                <p className="text-sm text-gray-300 mt-1 whitespace-pre-wrap break-all line-clamp-3">
                  {shortcut.content}
                </p>
              </div>
              <div className="flex gap-1 ml-2 shrink-0 opacity-0 group-hover:opacity-100 transition-opacity">
                <button
                  onClick={() => handleEdit(shortcut)}
                  className="w-7 h-7 flex items-center justify-center rounded bg-gray-800 hover:bg-gray-700 border border-gray-700 text-xs"
                  title="Edit"
                >
                  ✏️
                </button>
                <button
                  onClick={() => onDelete(shortcut.id)}
                  className="w-7 h-7 flex items-center justify-center rounded bg-gray-800 hover:bg-gray-700 border border-gray-700 text-xs"
                  title="Delete"
                >
                  🗑️
                </button>
              </div>
            </div>
          </div>
        ))}
      </div>

      {shortcuts.length === 0 && !showForm && (
        <div className="text-center text-gray-500 mt-8">
          <p className="text-3xl mb-2">⌨️</p>
          <p className="text-sm">No shortcuts yet</p>
          <p className="text-xs mt-1">Create one to start expanding text</p>
        </div>
      )}
    </div>
  );
}
