import type { ReactNode } from "react";
import Sidebar from "./Sidebar";
import { useStore } from "../store";

interface LayoutProps {
  children: ReactNode;
}

export default function Layout({ children }: LayoutProps) {
  const settings = useStore((s) => s.settings);

  return (
    <div className="flex flex-1 overflow-hidden bg-slate-50 min-h-0">
      <Sidebar />

      <div className="flex flex-col flex-1 min-w-0 overflow-hidden">
        {/* Top bar */}
        <header className="flex items-center justify-between px-4 py-2 bg-white border-b border-slate-200 shrink-0">
          <span className="font-semibold text-slate-800 tracking-tight">
            Cheshire CAT
          </span>
          {settings && (
            <span className="text-xs text-slate-500 font-mono bg-slate-100 px-2 py-0.5 rounded">
              {settings.sourceLang} → {settings.targetLang}
            </span>
          )}
        </header>

        {/* Main content */}
        <main className="flex-1 overflow-auto">{children}</main>
      </div>
    </div>
  );
}
