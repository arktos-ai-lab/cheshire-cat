import { useState } from "react";
import type { UpdateInfo } from "../api";

interface UpdateBannerProps {
  info: UpdateInfo;
}

export default function UpdateBanner({ info }: UpdateBannerProps) {
  const [dismissed, setDismissed] = useState(false);

  if (dismissed) return null;

  function openRelease() {
    // Open in the system browser via the shell plugin.
    // Tauri's shell.open is exposed via the plugin JS API.
    // We import dynamically to avoid a hard dependency when the plugin is absent.
    import("@tauri-apps/plugin-shell")
      .then(({ open }) => open(info.release_url))
      .catch(() => {
        window.open(info.release_url, "_blank", "noopener");
      });
  }

  return (
    <div className="flex items-center justify-between gap-3 px-4 py-2 bg-brand-600 text-white text-xs shrink-0">
      <div className="flex items-center gap-2 min-w-0">
        <span className="font-semibold shrink-0">Update available</span>
        <span className="text-brand-100 truncate">
          v{info.latest_version}
          {info.release_notes ? ` — ${info.release_notes}` : ""}
        </span>
      </div>
      <div className="flex items-center gap-2 shrink-0">
        <button
          className="px-2 py-0.5 rounded bg-white text-brand-700 font-medium hover:bg-brand-50 transition-colors"
          onClick={openRelease}
        >
          Download
        </button>
        <button
          className="text-brand-200 hover:text-white transition-colors"
          onClick={() => setDismissed(true)}
          title="Dismiss"
        >
          ✕
        </button>
      </div>
    </div>
  );
}
