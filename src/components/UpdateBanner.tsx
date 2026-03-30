import { useEffect, useState } from "react";
import { check } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";

export function UpdateBanner() {
  const [updateAvailable, setUpdateAvailable] = useState(false);
  const [version, setVersion] = useState("");
  const [installing, setInstalling] = useState(false);

  useEffect(() => {
    // Chequear si hay una actualización disponible al montar el componente
    check()
      .then((update) => {
        if (update) {
          setUpdateAvailable(true);
          setVersion(update.version);
        }
      })
      .catch(console.error);
  }, []);

  async function handleUpdate() {
    setInstalling(true);
    try {
      const update = await check();
      if (update) {
        await update.downloadAndInstall();
        await relaunch();
      }
    } catch (err) {
      console.error("Update failed:", err);
      setInstalling(false);
    }
  }

  if (!updateAvailable) return null;

  return (
    <div className="px-4 py-2 bg-blue-600/20 border-b border-blue-500/30 flex items-center justify-between">
      <p className="text-xs text-blue-300">
        Pane {version} is available!
      </p>
      <button
        onClick={handleUpdate}
        disabled={installing}
        className="text-xs bg-blue-600 hover:bg-blue-500 text-white px-3 py-1 rounded transition-colors disabled:opacity-50"
      >
        {installing ? "Installing..." : "Update now"}
      </button>
    </div>
  );
}
