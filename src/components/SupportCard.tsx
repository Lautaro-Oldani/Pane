import { invoke } from "@tauri-apps/api/core";

interface SupportCardProps {
  onClose: () => void;
}

export function SupportCard({ onClose }: SupportCardProps) {
  function openKofi() {
    invoke("plugin:opener|open_url", { url: "https://ko-fi.com/juanylato" });
  }

  return (
    <div className="flex-1 overflow-y-auto p-6">
      <div className="flex items-center justify-between mb-6">
        <h2 className="text-lg font-bold theme-text">Support Pane</h2>
        <button
          onClick={onClose}
          className="text-gray-400 hover:text-white text-sm px-2 py-1 rounded hover:bg-gray-800"
        >
          ← Back
        </button>
      </div>

      <div className="space-y-5">
        {/* Hero */}
        <div className="text-center py-4">
          <p className="text-4xl mb-3">☕</p>
          <h3 className="text-xl font-bold theme-text">Buy me a coffee</h3>
          <p className="text-sm theme-text-muted mt-1">Support indie development</p>
        </div>

        {/* Message */}
        <div className="space-y-3 text-sm theme-text-secondary leading-relaxed">
          <p>
            Hey! I'm <strong className="theme-text">juanylato</strong>, the developer behind Pane.
            I built this app because I was tired of losing things I copied — and I wanted to
            share it with everyone for <strong className="theme-text">free</strong>.
          </p>
          <p>
            Pane is and will remain free. No ads, no tracking, no data collection.
            Just a tool that works.
          </p>
          <p>
            But building and maintaining it takes a lot of time and energy. If Pane
            saves you even a few minutes a day, consider buying me a coffee — it
            genuinely helps me keep working on this and building more tools for the community.
          </p>
        </div>

        {/* CTA Button */}
        <button
          onClick={openKofi}
          className="w-full py-3 px-4 rounded-xl bg-[#FF5E5B] hover:bg-[#ff4744] text-white font-semibold text-sm transition-colors flex items-center justify-center gap-2 shadow-lg shadow-red-500/20"
        >
          <span className="text-lg">☕</span>
          Support Pane on Ko-fi
        </button>

        {/* Stats / social proof */}
        <p className="text-center text-xs theme-text-muted">
          Every coffee counts. Thank you for being awesome!
        </p>
      </div>
    </div>
  );
}
