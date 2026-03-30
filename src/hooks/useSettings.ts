import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

export interface AppSettings {
  history_limit: string;
  auto_clear_days: string;
  theme: string;
  hotkey: string;
  autostart: string;
}

const DEFAULTS: AppSettings = {
  history_limit: "500",
  auto_clear_days: "30",
  theme: "dark",
  hotkey: "Ctrl+Shift+V",
  autostart: "false",
};

export function useSettings() {
  const [settings, setSettings] = useState<AppSettings>(DEFAULTS);
  const [loaded, setLoaded] = useState(false);

  useEffect(() => {
    invoke<[string, string][]>("get_all_settings").then((pairs) => {
      const map: Record<string, string> = {};
      for (const [k, v] of pairs) map[k] = v;
      setSettings({ ...DEFAULTS, ...map });
      setLoaded(true);
    }).catch(console.error);
  }, []);

  const updateSetting = useCallback(async (key: keyof AppSettings, value: string) => {
    await invoke("set_setting", { key, value });
    setSettings((prev) => ({ ...prev, [key]: value }));
  }, []);

  return { settings, loaded, updateSetting };
}
