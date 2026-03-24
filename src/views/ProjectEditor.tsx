import clsx from "clsx";
import { useCallback, useEffect, useRef, useState } from "react";
import {
  aiSuggest,
  formatExportBilingualDocx,
  formatExportCsv,
  formatExportDocx,
  formatExportJson,
  formatExportPo,
  formatExportXliff,
  formatImport,
  glossaryLookup,
  openSourceFile,
  saveBilingualDocxFile,
  saveCsvFile,
  saveDocxFile,
  saveJsonFile,
  savePoFile,
  saveXliffFile,
  tmAddRecord,
  tmSearch,
} from "../api";
import MatchPanel from "../components/MatchPanel";
import { useStore } from "../store";
import type { ProjectSegment, SourceUnit } from "../types";
import { runQaChecks } from "../utils/qa";

// ── Segment status badge ──────────────────────────────────────────────────────

function StatusBadge({ status }: { status: ProjectSegment["status"] }) {
  if (status === "confirmed") {
    return (
      <span className="text-emerald-600 font-bold text-xs" title="Confirmed">
        ✓
      </span>
    );
  }
  if (status === "draft") {
    return (
      <span className="text-amber-500 font-bold text-xs" title="Draft">
        ~
      </span>
    );
  }
  return (
    <span className="text-slate-300 text-xs" title="Untranslated">
      ·
    </span>
  );
}

// ── Progress bar ──────────────────────────────────────────────────────────────

function ProgressBar({
  segments,
}: {
  segments: ProjectSegment[];
}) {
  const total = segments.length;
  const confirmed = segments.filter((s) => s.status === "confirmed").length;
  const drafted = segments.filter((s) => s.status === "draft").length;
  const pctConfirmed = total ? (confirmed / total) * 100 : 0;
  const pctDraft = total ? (drafted / total) * 100 : 0;

  return (
    <div className="flex items-center gap-3 text-xs text-slate-500">
      <div className="flex-1 h-1.5 bg-slate-200 rounded-full overflow-hidden flex">
        <div
          className="h-full bg-emerald-500 transition-all"
          style={{ width: `${pctConfirmed}%` }}
        />
        <div
          className="h-full bg-amber-400 transition-all"
          style={{ width: `${pctDraft}%` }}
        />
      </div>
      <span className="shrink-0 tabular-nums">
        {confirmed}/{total}
      </span>
    </div>
  );
}

// ── Main component ────────────────────────────────────────────────────────────

const DEBOUNCE_MS = 300;

export default function ProjectEditor() {
  const segments = useStore((s) => s.projectSegments);
  const activeIndex = useStore((s) => s.activeSegmentIndex);
  const fileName = useStore((s) => s.projectFileName);
  const sourceLang = useStore((s) => s.projectSourceLang);
  const targetLang = useStore((s) => s.projectTargetLang);
  const tmMatches = useStore((s) => s.projectTmMatches);
  const glossaryHits = useStore((s) => s.projectGlossaryHits);
  const aiDraft = useStore((s) => s.projectAiDraft);
  const isAiLoading = useStore((s) => s.isProjectAiLoading);
  const isSearching = useStore((s) => s.isProjectSearching);
  const settings = useStore((s) => s.settings);

  const sourcePath = useStore((s) => s.projectSourcePath);
  const openProject = useStore((s) => s.openProject);
  const setActiveSegmentIndex = useStore((s) => s.setActiveSegmentIndex);
  const updateSegmentTarget = useStore((s) => s.updateSegmentTarget);
  const confirmSegmentAt = useStore((s) => s.confirmSegmentAt);
  const setProjectTmMatches = useStore((s) => s.setProjectTmMatches);
  const setProjectGlossaryHits = useStore((s) => s.setProjectGlossaryHits);
  const setProjectAiDraft = useStore((s) => s.setProjectAiDraft);
  const setIsProjectAiLoading = useStore((s) => s.setIsProjectAiLoading);
  const setIsProjectSearching = useStore((s) => s.setIsProjectSearching);

  const activeSeg = segments[activeIndex] ?? null;
  const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const aiAbortRef = useRef<AbortController | null>(null);
  const targetRef = useRef<HTMLTextAreaElement>(null);
  const [opError, setOpError] = useState<string | null>(null);

  function showError(msg: string) {
    setOpError(msg);
    setTimeout(() => setOpError(null), 4000);
  }

  // ── Open file ──────────────────────────────────────────────────────────────

  async function handleOpen() {
    const path = await openSourceFile();
    if (!path) return;

    try {
      const result = await formatImport(path);
      const segments: ProjectSegment[] = (result.units as SourceUnit[]).map(
        (u) => ({
          id: u.id,
          source: u.source,
          target: u.target ?? "",
          status: u.target?.trim() ? "draft" : "untranslated",
          note: u.note,
        })
      );
      const baseName = path.split(/[\\/]/).pop() ?? path;
      openProject(
        segments,
        baseName,
        path,
        result.source_lang ?? settings?.sourceLang ?? "",
        result.target_lang ?? settings?.targetLang ?? ""
      );
    } catch (err) {
      showError(`Import failed: ${err}`);
    }
  }

  // ── Export ─────────────────────────────────────────────────────────────────

  async function handleExportXliff() {
    if (!segments.length) return;
    const base = fileName?.replace(/\.[^.]+$/, "") ?? "project";
    const path = await saveXliffFile(base);
    if (!path) return;

    const units: SourceUnit[] = segments.map((s) => ({
      id: s.id,
      source: s.source,
      target: s.target || null,
      note: s.note,
    }));

    try {
      await formatExportXliff(
        path,
        units,
        sourceLang ?? settings?.sourceLang ?? "",
        targetLang ?? settings?.targetLang ?? ""
      );
    } catch (err) {
      showError(`XLIFF export failed: ${err}`);
    }
  }

  async function handleExportDocx() {
    if (!segments.length || !fileName) return;
    const base = fileName.replace(/\.[^.]+$/, "");
    const destPath = await saveDocxFile(base);
    if (!destPath) return;

    // Build source→target map from all segments that have a translation.
    const translations: Record<string, string> = {};
    for (const seg of segments) {
      if (seg.target.trim()) {
        translations[seg.source] = seg.target;
      }
    }

    // Use the stored source path; only fall back to file picker for non-DOCX source.
    const src = sourcePath ?? (await openSourceFile());
    if (!src) return;

    try {
      await formatExportDocx(src, destPath, translations);
    } catch (err) {
      showError(`DOCX export failed: ${err}`);
    }
  }

  async function handleExportPo() {
    if (!segments.length) return;
    const base = fileName?.replace(/\.[^.]+$/, "") ?? "project";
    const path = await savePoFile(base);
    if (!path) return;
    const units: SourceUnit[] = segments.map((s) => ({
      id: s.id, source: s.source, target: s.target || null, note: s.note,
    }));
    try { await formatExportPo(path, units); } catch (err) { showError(`PO export failed: ${err}`); }
  }

  async function handleExportJson() {
    if (!segments.length) return;
    const base = fileName?.replace(/\.[^.]+$/, "") ?? "project";
    const path = await saveJsonFile(base);
    if (!path) return;
    const units: SourceUnit[] = segments.map((s) => ({
      id: s.id, source: s.source, target: s.target || null, note: s.note,
    }));
    try { await formatExportJson(path, units); } catch (err) { showError(`JSON export failed: ${err}`); }
  }

  async function handleExportCsv() {
    if (!segments.length) return;
    const base = fileName?.replace(/\.[^.]+$/, "") ?? "project";
    const path = await saveCsvFile(base);
    if (!path) return;
    const units: SourceUnit[] = segments.map((s) => ({
      id: s.id, source: s.source, target: s.target || null, note: s.note,
    }));
    try { await formatExportCsv(path, units); } catch (err) { showError(`CSV export failed: ${err}`); }
  }

  async function handleExportBilingualDocx() {
    if (!segments.length) return;
    const base = fileName?.replace(/\.[^.]+$/, "") ?? "project";
    const path = await saveBilingualDocxFile(base);
    if (!path) return;
    const units: SourceUnit[] = segments.map((s) => ({
      id: s.id, source: s.source, target: s.target || null, note: s.note,
    }));
    const sl = sourceLang ?? settings?.sourceLang ?? "";
    const tl = targetLang ?? settings?.targetLang ?? "";
    try {
      await formatExportBilingualDocx(path, units, sl, tl);
    } catch (err) {
      showError(`Bilingual DOCX export failed: ${err}`);
    }
  }

  // ── TM + glossary lookup when active segment changes ───────────────────────

  const lookupSegment = useCallback(
    (source: string) => {
      if (debounceRef.current) clearTimeout(debounceRef.current);
      aiAbortRef.current?.abort();

      if (!source.trim()) {
        setProjectTmMatches([]);
        setProjectGlossaryHits([]);
        setProjectAiDraft(null);
        return;
      }

      debounceRef.current = setTimeout(async () => {
        setIsProjectSearching(true);
        try {
          const [matches, hits] = await Promise.all([
            tmSearch(source),
            glossaryLookup(source),
          ]);
          setProjectTmMatches(matches);
          setProjectGlossaryHits(hits);

          // AI pre-fetch — fires in background
          if (!settings || settings.aiMode === "disabled") return;

          const ctrl = new AbortController();
          aiAbortRef.current = ctrl;
          setIsProjectAiLoading(true);
          setProjectAiDraft(null);

          aiSuggest({
            source,
            sourceLang: sourceLang ?? settings.sourceLang,
            targetLang: targetLang ?? settings.targetLang,
            tmMatches: matches.slice(0, 3).map((m) => ({
              source: m.record.source,
              target: m.record.target,
              score: Math.round(m.score * 100),
            })),
            glossaryHits: hits.map((h) => ({
              source_term: h.term.source_term,
              target_term: h.term.target_term,
            })),
          })
            .then((draft) => {
              if (!ctrl.signal.aborted) setProjectAiDraft(draft);
            })
            .catch(() => {})
            .finally(() => {
              if (!ctrl.signal.aborted) setIsProjectAiLoading(false);
            });
        } catch {
          // ignore
        } finally {
          setIsProjectSearching(false);
        }
      }, DEBOUNCE_MS);
    },
    [
      settings,
      sourceLang,
      targetLang,
      setProjectTmMatches,
      setProjectGlossaryHits,
      setProjectAiDraft,
      setIsProjectSearching,
      setIsProjectAiLoading,
    ]
  );

  useEffect(() => {
    if (activeSeg) {
      lookupSegment(activeSeg.source);
      // Focus the target textarea when segment changes
      requestAnimationFrame(() => targetRef.current?.focus());
    }
    return () => {
      if (debounceRef.current) clearTimeout(debounceRef.current);
      aiAbortRef.current?.abort();
    };
  }, [activeIndex]); // eslint-disable-line react-hooks/exhaustive-deps

  // ── Confirm + advance ──────────────────────────────────────────────────────

  async function handleConfirm() {
    if (!activeSeg || !activeSeg.target.trim()) return;
    confirmSegmentAt(activeIndex);
    // Save to TM
    try {
      await tmAddRecord(activeSeg.source, activeSeg.target);
    } catch {
      // TM save failure is non-fatal
    }
    // Advance to next untranslated segment
    const next = segments.findIndex(
      (s, i) => i > activeIndex && s.status !== "confirmed"
    );
    if (next !== -1) setActiveSegmentIndex(next);
  }

  // ── Keyboard shortcuts ─────────────────────────────────────────────────────

  function handleTargetKeyDown(e: React.KeyboardEvent) {
    if (e.key === "Enter" && (e.ctrlKey || e.metaKey)) {
      e.preventDefault();
      handleConfirm();
    }
  }

  // ── QA issues for active segment ──────────────────────────────────────────

  const qaIssues =
    activeSeg?.source && activeSeg?.target
      ? runQaChecks(activeSeg.source, activeSeg.target)
      : [];

  // ── Empty state ────────────────────────────────────────────────────────────

  if (!segments.length) {
    return (
      <div className="flex flex-col items-center justify-center h-full gap-4">
        <div className="text-center">
          <p className="text-lg font-semibold text-slate-700 mb-1">
            No file open
          </p>
          <p className="text-sm text-slate-400">
            Open a DOCX, XLIFF, PO, HTML, JSON, or plain text file to begin.
          </p>
        </div>
        <button className="btn-primary" onClick={handleOpen}>
          Open file…
        </button>
      </div>
    );
  }

  // ── Main UI ────────────────────────────────────────────────────────────────

  return (
    <div className="flex flex-col h-full overflow-hidden">
      {/* ── Toolbar ── */}
      <div className="flex items-center gap-3 px-4 py-2 bg-white border-b border-slate-200 shrink-0">
        <button className="btn-secondary py-1" onClick={handleOpen}>
          Open file…
        </button>

        <span className="text-sm font-medium text-slate-700 truncate min-w-0">
          {fileName}
        </span>

        {(sourceLang || targetLang) && (
          <span className="text-xs text-slate-500 font-mono bg-slate-100 px-2 py-0.5 rounded shrink-0">
            {sourceLang} → {targetLang}
          </span>
        )}

        <div className="flex-1 min-w-0 mx-2">
          <ProgressBar segments={segments} />
        </div>

        <div className="flex gap-2 shrink-0">
          <button
            className="btn-secondary py-1"
            title="Jump to next untranslated segment"
            onClick={() => {
              const next = segments.findIndex(
                (s, i) => i >= 0 && s.status === "untranslated"
              );
              if (next !== -1) setActiveSegmentIndex(next);
            }}
          >
            First untranslated ↓
          </button>
          <button className="btn-secondary py-1" onClick={handleExportXliff} title="Export as XLIFF 1.2">XLIFF</button>
          <button className="btn-secondary py-1" onClick={handleExportPo} title="Export as PO (Gettext)">PO</button>
          <button className="btn-secondary py-1" onClick={handleExportJson} title="Export as JSON">JSON</button>
          <button className="btn-secondary py-1" onClick={handleExportCsv} title="Export as CSV">CSV</button>
          <button className="btn-secondary py-1" onClick={handleExportBilingualDocx} title="Export bilingual DOCX (source + target table)">Bilingual</button>
          <button className="btn-primary py-1" onClick={handleExportDocx} title="Export as translated DOCX">DOCX</button>
        </div>
      </div>

      {/* ── Error banner ── */}
      {opError && (
        <div className="shrink-0 px-4 py-2 text-xs text-red-700 bg-red-50 border-b border-red-200">
          {opError}
        </div>
      )}

      {/* ── Body ── */}
      <div className="flex flex-1 overflow-hidden">
        {/* ── Segment table ── */}
        <div className="flex flex-col w-96 border-r border-slate-200 bg-white overflow-hidden shrink-0">
          <div className="flex-1 overflow-y-auto">
            <table className="w-full text-xs">
              <thead className="bg-slate-50 border-b border-slate-200 sticky top-0 z-10">
                <tr>
                  <th className="px-2 py-1.5 text-left font-medium text-slate-500 w-6">
                    #
                  </th>
                  <th className="px-2 py-1.5 text-left font-medium text-slate-500">
                    Source
                  </th>
                  <th className="px-2 py-1.5 text-left font-medium text-slate-500">
                    Target
                  </th>
                  <th className="w-5 px-1" />
                </tr>
              </thead>
              <tbody className="divide-y divide-slate-100">
                {segments.map((seg, i) => (
                  <tr
                    key={seg.id}
                    className={clsx(
                      "cursor-pointer transition-colors",
                      i === activeIndex
                        ? "bg-brand-50 outline outline-1 outline-brand-300"
                        : "hover:bg-slate-50"
                    )}
                    onClick={() => setActiveSegmentIndex(i)}
                  >
                    <td className="px-2 py-1.5 text-slate-400 tabular-nums">
                      {i + 1}
                    </td>
                    <td className="px-2 py-1.5 text-slate-600 max-w-0 truncate">
                      {seg.source}
                    </td>
                    <td
                      className={clsx(
                        "px-2 py-1.5 max-w-0 truncate",
                        seg.status === "confirmed"
                          ? "text-slate-800"
                          : "text-slate-400 italic"
                      )}
                    >
                      {seg.target || "—"}
                    </td>
                    <td className="px-1 py-1.5 text-center">
                      <StatusBadge status={seg.status} />
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>

        {/* ── Editor + right panel ── */}
        <div className="flex flex-1 overflow-hidden">
          {/* Active segment editor */}
          <div className="flex flex-col flex-1 min-w-0 p-4 gap-3 overflow-hidden">
            {activeSeg ? (
              <>
                {/* Source (read-only) */}
                <div className="flex flex-col shrink-0">
                  <label className="label">
                    Source
                    {activeSeg.note && (
                      <span className="ml-2 text-slate-400 font-normal italic">
                        {activeSeg.note}
                      </span>
                    )}
                  </label>
                  <div className="segment-area min-h-[5rem] text-sm text-slate-700 bg-slate-50 select-text cursor-text overflow-auto">
                    {activeSeg.source}
                  </div>
                </div>

                {/* Target (editable) */}
                <div className="flex flex-col flex-1 min-h-0">
                  <label className="label">Target</label>
                  <textarea
                    ref={targetRef}
                    className="segment-area flex-1 text-sm"
                    value={activeSeg.target}
                    onChange={(e) =>
                      updateSegmentTarget(activeIndex, e.target.value)
                    }
                    onKeyDown={handleTargetKeyDown}
                    placeholder="Type translation here… (Ctrl+Enter to confirm)"
                  />
                </div>

                {/* QA warnings */}
                {qaIssues.length > 0 && (
                  <div className="shrink-0 space-y-1">
                    {qaIssues.map((issue, i) => (
                      <div
                        key={i}
                        className="flex items-start gap-2 text-xs text-amber-700 bg-amber-50 border border-amber-200 rounded-md px-3 py-1.5"
                      >
                        <span className="shrink-0 font-bold">!</span>
                        <span>{issue.message}</span>
                      </div>
                    ))}
                  </div>
                )}

                {/* Actions */}
                <div className="flex items-center justify-between shrink-0">
                  <div className="flex gap-2">
                    <button
                      className="btn-secondary py-1"
                      disabled={activeIndex === 0}
                      onClick={() => setActiveSegmentIndex(activeIndex - 1)}
                    >
                      ← Prev
                    </button>
                    <button
                      className="btn-secondary py-1"
                      disabled={activeIndex >= segments.length - 1}
                      onClick={() => setActiveSegmentIndex(activeIndex + 1)}
                    >
                      Next →
                    </button>
                  </div>
                  <button
                    className="btn-primary"
                    disabled={!activeSeg.target.trim()}
                    onClick={handleConfirm}
                    title="Ctrl+Enter"
                  >
                    Confirm ↵
                  </button>
                </div>
              </>
            ) : (
              <div className="flex items-center justify-center h-full text-sm text-slate-400">
                Select a segment to edit.
              </div>
            )}
          </div>

          {/* Right panel: TM + glossary + AI */}
          <aside className="w-72 flex flex-col border-l border-slate-200 bg-white overflow-hidden shrink-0">
            {/* TM Matches */}
            <div className="flex flex-col flex-1 min-h-0 overflow-hidden">
              <div className="px-3 py-2 border-b border-slate-100 shrink-0">
                <h2 className="text-xs font-semibold text-slate-500 uppercase tracking-wide">
                  TM Matches
                </h2>
              </div>
              <div className="flex-1 overflow-y-auto">
                <MatchPanel
                  matches={tmMatches}
                  isLoading={isSearching}
                  onApply={(match) => {
                    updateSegmentTarget(activeIndex, match.record.target);
                    targetRef.current?.focus();
                  }}
                />
              </div>
            </div>

            {/* Glossary Hits */}
            {glossaryHits.length > 0 && (
              <div className="border-t border-slate-200 shrink-0">
                <div className="px-3 py-2 border-b border-slate-100">
                  <h2 className="text-xs font-semibold text-slate-500 uppercase tracking-wide">
                    Glossary
                  </h2>
                </div>
                <ul className="divide-y divide-slate-100 max-h-32 overflow-y-auto">
                  {glossaryHits.map((h) => (
                    <li key={h.term.id} className="px-3 py-1.5">
                      <div className="flex items-center justify-between gap-2">
                        <span className="text-xs font-medium text-slate-700">
                          {h.term.source_term}
                        </span>
                        <span className="text-xs text-brand-600 font-medium">
                          {h.term.target_term}
                        </span>
                      </div>
                      {h.term.forbidden && (
                        <span className="text-xs text-red-500 font-medium">
                          Forbidden
                        </span>
                      )}
                    </li>
                  ))}
                </ul>
              </div>
            )}

            {/* AI Draft */}
            {settings && settings.aiMode !== "disabled" && (isAiLoading || aiDraft) && (
              <div className="border-t border-slate-200 shrink-0">
                <div className="px-3 py-2 border-b border-slate-100 flex items-center justify-between">
                  <h2 className="text-xs font-semibold text-slate-500 uppercase tracking-wide">
                    AI Draft
                  </h2>
                  {isAiLoading && (
                    <span className="text-xs text-slate-400 animate-pulse">
                      Generating…
                    </span>
                  )}
                </div>
                {aiDraft && (
                  <div className="p-3">
                    <p className="text-xs text-slate-700 leading-snug mb-2">
                      {aiDraft.text}
                    </p>
                    {aiDraft.is_fallback && (
                      <p className="text-xs text-amber-600 mb-2">
                        Model returned an empty response.
                      </p>
                    )}
                    <button
                      className="btn-secondary py-1 px-2 text-xs w-full justify-center"
                      onClick={() => {
                        updateSegmentTarget(activeIndex, aiDraft.text);
                        targetRef.current?.focus();
                      }}
                    >
                      Apply draft
                    </button>
                  </div>
                )}
              </div>
            )}
          </aside>
        </div>
      </div>
    </div>
  );
}
