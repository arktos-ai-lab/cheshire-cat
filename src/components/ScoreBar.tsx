import clsx from "clsx";

interface ScoreBarProps {
  score: number;
  matchType: "Exact" | "Fuzzy" | "ContextMatch";
}

function scoreColor(score: number): string {
  if (score >= 1.0) return "bg-emerald-500";
  if (score >= 0.85) return "bg-sky-500";
  if (score >= 0.70) return "bg-amber-400";
  return "bg-slate-300";
}

function scoreLabel(score: number, matchType: string): string {
  if (matchType === "Exact") return "100%";
  return `${Math.round(score * 100)}%`;
}

export default function ScoreBar({ score, matchType }: ScoreBarProps) {
  const pct = Math.min(1, score);
  const colorClass = matchType === "Exact" ? "bg-emerald-500" : scoreColor(score);
  const label = scoreLabel(score, matchType);

  return (
    <div className="flex items-center gap-2 shrink-0">
      <div className="w-16 h-1.5 bg-slate-200 rounded-full overflow-hidden">
        <div
          className={clsx("h-full rounded-full transition-all", colorClass)}
          style={{ width: `${pct * 100}%` }}
        />
      </div>
      <span
        className={clsx(
          "text-xs font-mono font-semibold w-9 text-right",
          matchType === "Exact"
            ? "text-emerald-600"
            : score >= 0.85
            ? "text-sky-600"
            : score >= 0.70
            ? "text-amber-600"
            : "text-slate-500"
        )}
      >
        {label}
      </span>
    </div>
  );
}
