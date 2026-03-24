import clsx from "clsx";
import { useStore } from "../store";
import type { View } from "../types";
import felixLogo from "../assets/cheshire_logo.png";

const NAV_ITEMS: { label: string; view: View; icon: string }[] = [
  { label: "Workbench", view: "workbench", icon: "✦" },
  { label: "Project", view: "project", icon: "⊕" },
  { label: "Memory", view: "memory", icon: "⊞" },
  { label: "Glossary", view: "glossary", icon: "⊟" },
  { label: "Settings", view: "settings", icon: "⚙" },
  { label: "Help", view: "help", icon: "?" },
];

export default function Sidebar() {
  const currentView = useStore((s) => s.view);
  const setView = useStore((s) => s.setView);

  return (
    <nav className="w-14 flex flex-col items-center py-4 gap-1 bg-slate-900 shrink-0">
      <img src={felixLogo} alt="Felix" className="w-9 h-9 mb-3" />
      {NAV_ITEMS.map(({ label, view, icon }) => (
        <button
          key={view}
          title={label}
          onClick={() => setView(view)}
          className={clsx(
            "w-10 h-10 flex items-center justify-center rounded-lg text-lg transition-colors",
            currentView === view
              ? "bg-brand-600 text-white"
              : "text-slate-400 hover:text-white hover:bg-slate-700"
          )}
        >
          {icon}
        </button>
      ))}
    </nav>
  );
}
