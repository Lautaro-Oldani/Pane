import type { AppSettings } from "../hooks/useSettings";

interface SettingsProps {
  settings: AppSettings;
  onUpdate: (key: keyof AppSettings, value: string) => void;
  onClose: () => void;
}

export function Settings({ settings, onUpdate, onClose }: SettingsProps) {
  return (
    <div className="flex-1 overflow-y-auto p-6 space-y-6">
      <div className="flex items-center justify-between">
        <h2 className="text-lg font-bold text-white">Settings</h2>
        <button
          onClick={onClose}
          className="text-gray-400 hover:text-white text-sm px-2 py-1 rounded hover:bg-gray-800"
        >
          ← Back
        </button>
      </div>

      {/* History Limit */}
      <SettingRow
        label="History limit"
        description="Maximum number of clips to keep"
      >
        <select
          value={settings.history_limit}
          onChange={(e) => onUpdate("history_limit", e.target.value)}
          className="bg-gray-800 border border-gray-700 rounded px-3 py-1.5 text-sm text-gray-200 focus:outline-none focus:border-gray-500"
        >
          <option value="100">100</option>
          <option value="500">500</option>
          <option value="1000">1,000</option>
          <option value="5000">5,000</option>
        </select>
      </SettingRow>

      {/* Auto-clear */}
      <SettingRow
        label="Auto-clear after"
        description="Delete clips older than this (0 = never)"
      >
        <select
          value={settings.auto_clear_days}
          onChange={(e) => onUpdate("auto_clear_days", e.target.value)}
          className="bg-gray-800 border border-gray-700 rounded px-3 py-1.5 text-sm text-gray-200 focus:outline-none focus:border-gray-500"
        >
          <option value="0">Never</option>
          <option value="7">7 days</option>
          <option value="14">14 days</option>
          <option value="30">30 days</option>
          <option value="60">60 days</option>
          <option value="90">90 days</option>
        </select>
      </SettingRow>

      {/* Theme */}
      <SettingRow
        label="Theme"
        description="App appearance"
      >
        <select
          value={settings.theme}
          onChange={(e) => onUpdate("theme", e.target.value)}
          className="bg-gray-800 border border-gray-700 rounded px-3 py-1.5 text-sm text-gray-200 focus:outline-none focus:border-gray-500"
        >
          <option value="dark">Dark</option>
          <option value="light">Light</option>
          <option value="system">System</option>
        </select>
      </SettingRow>

      {/* Hotkey */}
      <SettingRow
        label="Global hotkey"
        description="Shortcut to toggle Pane"
      >
        <span className="text-sm text-gray-400 bg-gray-800 border border-gray-700 rounded px-3 py-1.5">
          {settings.hotkey}
        </span>
      </SettingRow>

      {/* Autostart */}
      <SettingRow
        label="Start on boot"
        description="Launch Pane when Windows starts"
      >
        <button
          onClick={() => onUpdate("autostart", settings.autostart === "true" ? "false" : "true")}
          className={`relative w-11 h-6 rounded-full transition-colors ${
            settings.autostart === "true" ? "bg-blue-600" : "bg-gray-700"
          }`}
        >
          <span
            className={`absolute top-0.5 left-0.5 w-5 h-5 bg-white rounded-full transition-transform ${
              settings.autostart === "true" ? "translate-x-5" : ""
            }`}
          />
        </button>
      </SettingRow>
    </div>
  );
}

function SettingRow({
  label,
  description,
  children,
}: {
  label: string;
  description: string;
  children: React.ReactNode;
}) {
  return (
    <div className="flex items-center justify-between py-2">
      <div>
        <p className="text-sm font-medium text-gray-200">{label}</p>
        <p className="text-xs text-gray-500">{description}</p>
      </div>
      {children}
    </div>
  );
}
