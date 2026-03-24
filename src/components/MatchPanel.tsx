import clsx from "clsx";
import ScoreBar from "./ScoreBar";
import { useStore } from "../store";
import type { TmMatch } from "../types";

interface MatchPanelProps {
  matches: TmMatch[];
  isLoading: boolean;
  /** Override the default store-based apply action (used in ProjectEditor). */
  onApply?: (match: TmMatch) => void;
}

export default function MatchPanel({ matches, isLoading, onApply }: MatchPanelProps) {
  const applyMatch = useStore((s) => s.applyMatch);
  const handleApply = onApply ?? applyMatch;

  if (isLoading) {
    return (
      <div className="p-3 text-xs text-slate-400 animate-pulse">
        Searching…
      </div>
    );
  }

  if (matches.length === 0) {
    return (
      <div className="p-3 text-xs text-slate-400">No matches found.</div>
    );
  }

  return (
    <ul className="divide-y divide-slate-100">
      {matches.map((m) => (
        <li
          key={m.record.id}
          className={clsx(
            "p-3 cursor-pointer hover:bg-slate-50 transition-colors group"
          )}
          onClick={() => handleApply(m)}
          title="Click to apply this match"
        >
          <div className="flex items-start justify-between gap-2 mb-1.5">
            <ScoreBar score={m.score} matchType={m.match_type} />
            <span className="text-xs text-slate-400 group-hover:text-brand-600 transition-colors">
              Apply ↵
            </span>
          </div>

          <p className="text-xs text-slate-500 leading-snug mb-1 line-clamp-2">
            {m.record.source}
          </p>
          <p className="text-xs text-slate-800 leading-snug font-medium line-clamp-2">
            {m.record.target}
          </p>

          {m.record.metadata.domain && (
            <span className="mt-1 inline-block text-xs bg-slate-100 text-slate-500 px-1.5 py-0.5 rounded">
              {m.record.metadata.domain}
            </span>
          )}
        </li>
      ))}
    </ul>
  );
}
