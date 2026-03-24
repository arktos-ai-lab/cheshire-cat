import { useCallback, useEffect, useRef, useState } from "react";
import {
  aiSuggest,
  glossaryLookup,
  officeExcelIsRunning,
  officeGetExcelSelection,
  officeGetPptSelection,
  officeGetWordSelection,
  officeInsertIntoExcel,
  officeInsertIntoPpt,
  officeInsertIntoWord,
  officePptIsRunning,
  officeWordIsRunning,
  settingsSet,
  tmAddRecord,
  tmSearch,
} from "../api";
import MatchPanel from "../components/MatchPanel";
import { useStore } from "../store";
import { runQaChecks } from "../utils/qa";

const DEBOUNCE_MS = 300;

export default function Workbench() {
  const sourceText = useStore((s) => s.sourceText);
  const targetText = useStore((s) => s.targetText);
  const tmMatches = useStore((s) => s.tmMatches);
  const glossaryHits = useStore((s) => s.glossaryHits);
  const qaIssues = useStore((s) => s.qaIssues);
  const isSearching = useStore((s) => s.isSearching);
  const aiDraft = useStore((s) => s.aiDraft);
  const isAiLoading = useStore((s) => s.isAiLoading);
  const settings = useStore((s) => s.settings);

  const setSourceText = useStore((s) => s.setSourceText);
  const setTargetText = useStore((s) => s.setTargetText);
  const setTmMatches = useStore((s) => s.setTmMatches);
  const setGlossaryHits = useStore((s) => s.setGlossaryHits);
  const setQaIssues = useStore((s) => s.setQaIssues);
  const setIsSearching = useStore((s) => s.setIsSearching);
  const setAiDraft = useStore((s) => s.setAiDraft);
  const setIsAiLoading = useStore((s) => s.setIsAiLoading);

  // ── Office COM state ───────────────────────────────────────────────────────
  const [wordRunning, setWordRunning]   = useState(false);
  const [excelRunning, setExcelRunning] = useState(false);
  const [pptRunning, setPptRunning]     = useState(false);
  const [officeMsg, setOfficeMsg]       = useState<string | null>(null);
  const [isInserting, setIsInserting]   = useState(false);

  // Poll for Office app availability once on mount, then every 5 s.
  useEffect(() => {
    let cancelled = false;
    async function poll() {
      const [word, excel, ppt] = await Promise.all([
        officeWordIsRunning().catch(() => false),
        officeExcelIsRunning().catch(() => false),
        officePptIsRunning().catch(() => false),
      ]);
      if (!cancelled) {
        setWordRunning(word);
        setExcelRunning(excel);
        setPptRunning(ppt);
      }
    }
    poll();
    const id = setInterval(poll, 5000);
    return () => { cancelled = true; clearInterval(id); };
  }, []);

  async function handleGetFrom(app: "word" | "excel" | "ppt") {
    setOfficeMsg(null);
    try {
      const fn = app === "word" ? officeGetWordSelection
               : app === "excel" ? officeGetExcelSelection
               : officeGetPptSelection;
      const text = await fn();
      if (text) { setSourceText(text); }
      else { setOfficeMsg(`No text selected in ${app === "ppt" ? "PowerPoint" : app === "excel" ? "Excel" : "Word"}.`); }
    } catch {
      setOfficeMsg("Could not read from Office application.");
    }
  }

  async function handleInsertInto(app: "word" | "excel" | "ppt") {
    if (!targetText.trim()) return;
    setIsInserting(true);
    setOfficeMsg(null);
    try {
      const fn = app === "word" ? officeInsertIntoWord
               : app === "excel" ? officeInsertIntoExcel
               : officeInsertIntoPpt;
      await fn(targetText);
      setOfficeMsg(`Inserted into ${app === "ppt" ? "PowerPoint" : app === "excel" ? "Excel" : "Word"}.`);
      setTimeout(() => setOfficeMsg(null), 2500);
    } catch {
      setOfficeMsg("Could not insert into Office application.");
    } finally {
      setIsInserting(false);
    }
  }

  // ── Inline language pair switcher ─────────────────────────────────────────
  const setSettings = useStore((s) => s.setSettings);
  const [editingSrc, setEditingSrc] = useState(false);
  const [editingTgt, setEditingTgt] = useState(false);
  const [draftSrc, setDraftSrc]     = useState("");
  const [draftTgt, setDraftTgt]     = useState("");

  function startEditLang(which: "src" | "tgt") {
    if (!settings) return;
    if (which === "src") { setDraftSrc(settings.sourceLang); setEditingSrc(true); }
    else                 { setDraftTgt(settings.targetLang); setEditingTgt(true); }
  }

  async function commitLang(which: "src" | "tgt") {
    if (!settings) return;
    const updated = {
      ...settings,
      sourceLang: which === "src" ? draftSrc.trim() || settings.sourceLang : settings.sourceLang,
      targetLang: which === "tgt" ? draftTgt.trim() || settings.targetLang : settings.targetLang,
    };
    setEditingSrc(false);
    setEditingTgt(false);
    await settingsSet(updated).catch(() => {});
    setSettings(updated);
  }

  const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const aiAbortRef = useRef<AbortController | null>(null);

  // Debounced TM + glossary lookup, then AI pre-fetch in background.
  const lookupSource = useCallback(
    (text: string) => {
      if (debounceRef.current) clearTimeout(debounceRef.current);
      aiAbortRef.current?.abort();

      if (!text.trim()) {
        setTmMatches([]);
        setGlossaryHits([]);
        setAiDraft(null);
        return;
      }

      debounceRef.current = setTimeout(async () => {
        setIsSearching(true);
        try {
          const [matches, hits] = await Promise.all([
            tmSearch(text),
            glossaryLookup(text),
          ]);
          setTmMatches(matches);
          setGlossaryHits(hits);

          if (!settings || settings.aiMode === "disabled") return;

          const ctrl = new AbortController();
          aiAbortRef.current = ctrl;
          setIsAiLoading(true);
          setAiDraft(null);

          aiSuggest({
            source: text,
            sourceLang: settings.sourceLang,
            targetLang: settings.targetLang,
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
              if (!ctrl.signal.aborted) setAiDraft(draft);
            })
            .catch(() => {})
            .finally(() => {
              if (!ctrl.signal.aborted) setIsAiLoading(false);
            });
        } catch {
          // ignore — user can retry by editing
        } finally {
          setIsSearching(false);
        }
      }, DEBOUNCE_MS);
    },
    [settings, setTmMatches, setGlossaryHits, setIsSearching, setAiDraft, setIsAiLoading]
  );

  useEffect(() => {
    lookupSource(sourceText);
    return () => {
      if (debounceRef.current) clearTimeout(debounceRef.current);
      aiAbortRef.current?.abort();
    };
  }, [sourceText, lookupSource]);

  useEffect(() => {
    if (sourceText && targetText) {
      setQaIssues(runQaChecks(sourceText, targetText));
    } else {
      setQaIssues([]);
    }
  }, [sourceText, targetText, setQaIssues]);

  async function confirmSegment() {
    if (!sourceText.trim() || !targetText.trim()) return;
    try {
      await tmAddRecord(sourceText, targetText);
      setSourceText("");
      setTargetText("");
    } catch {
      setOfficeMsg("Failed to save segment to TM.");
      setTimeout(() => setOfficeMsg(null), 3000);
    }
  }

  return (
    <div className="flex h-full overflow-hidden">
      {/* ── Left: source + target editors ── */}
      <div className="flex flex-col flex-1 min-w-0 p-4 gap-3 overflow-hidden">

        {/* Language pair pill + Office buttons row */}
        <div className="flex items-center justify-between shrink-0 gap-2">
          {/* Language pair quick-switcher */}
          <div className="flex items-center gap-1 text-xs text-slate-500">
            <span className="text-slate-400">Pair:</span>
            {editingSrc ? (
              <input
                autoFocus
                className="w-14 border border-brand-400 rounded px-1 py-0.5 text-xs font-mono text-slate-700 outline-none"
                value={draftSrc}
                onChange={(e) => setDraftSrc(e.target.value)}
                onBlur={() => commitLang("src")}
                onKeyDown={(e) => { if (e.key === "Enter") commitLang("src"); if (e.key === "Escape") setEditingSrc(false); }}
              />
            ) : (
              <button
                className="font-mono text-slate-700 hover:text-brand-600 hover:underline"
                title="Click to edit source language"
                onClick={() => startEditLang("src")}
              >
                {settings?.sourceLang || "??"}
              </button>
            )}
            <span>→</span>
            {editingTgt ? (
              <input
                autoFocus
                className="w-14 border border-brand-400 rounded px-1 py-0.5 text-xs font-mono text-slate-700 outline-none"
                value={draftTgt}
                onChange={(e) => setDraftTgt(e.target.value)}
                onBlur={() => commitLang("tgt")}
                onKeyDown={(e) => { if (e.key === "Enter") commitLang("tgt"); if (e.key === "Escape") setEditingTgt(false); }}
              />
            ) : (
              <button
                className="font-mono text-slate-700 hover:text-brand-600 hover:underline"
                title="Click to edit target language"
                onClick={() => startEditLang("tgt")}
              >
                {settings?.targetLang || "??"}
              </button>
            )}
          </div>

          {/* Office app buttons */}
          <div className="flex items-center gap-1">
            {wordRunning && (
              <button className="btn-secondary py-0.5 px-2 text-xs" onClick={() => handleGetFrom("word")} title="Get selected text from Word">
                ← Word
              </button>
            )}
            {excelRunning && (
              <button className="btn-secondary py-0.5 px-2 text-xs" onClick={() => handleGetFrom("excel")} title="Get selected cell from Excel">
                ← Excel
              </button>
            )}
            {pptRunning && (
              <button className="btn-secondary py-0.5 px-2 text-xs" onClick={() => handleGetFrom("ppt")} title="Get selected text from PowerPoint">
                ← PPT
              </button>
            )}
          </div>
        </div>

        {/* Source */}
        <div className="flex flex-col flex-1 min-h-0">
          <label className="label mb-1">Source</label>
          <textarea
            className="segment-area flex-1"
            placeholder="Paste or type source text…"
            value={sourceText}
            onChange={(e) => setSourceText(e.target.value)}
            spellCheck={false}
          />
        </div>

        {/* Target */}
        <div className="flex flex-col flex-1 min-h-0">
          <div className="flex items-center justify-between mb-1">
            <label className="label mb-0">Target</label>
            <div className="flex items-center gap-1">
              {wordRunning && (
                <button
                  className="btn-secondary py-0.5 px-2 text-xs"
                  disabled={!targetText.trim() || isInserting}
                  onClick={() => handleInsertInto("word")}
                  title="Insert translation into Word"
                >
                  Word →
                </button>
              )}
              {excelRunning && (
                <button
                  className="btn-secondary py-0.5 px-2 text-xs"
                  disabled={!targetText.trim() || isInserting}
                  onClick={() => handleInsertInto("excel")}
                  title="Insert translation into active Excel cell"
                >
                  Excel →
                </button>
              )}
              {pptRunning && (
                <button
                  className="btn-secondary py-0.5 px-2 text-xs"
                  disabled={!targetText.trim() || isInserting}
                  onClick={() => handleInsertInto("ppt")}
                  title="Insert translation into PowerPoint selection"
                >
                  PPT →
                </button>
              )}
            </div>
          </div>
          <textarea
            className="segment-area flex-1"
            placeholder="Type translation here…"
            value={targetText}
            onChange={(e) => setTargetText(e.target.value)}
          />
        </div>

        {/* Office feedback */}
        {officeMsg && (
          <div className="shrink-0 text-xs text-slate-600 bg-slate-100 border border-slate-200 rounded-md px-3 py-1.5">
            {officeMsg}
          </div>
        )}

        {/* QA issues */}
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
        <div className="flex items-center justify-end gap-2 shrink-0">
          <button
            className="btn-secondary"
            onClick={() => {
              setSourceText("");
              setTargetText("");
            }}
          >
            Clear
          </button>
          <button
            className="btn-primary"
            disabled={!sourceText.trim() || !targetText.trim()}
            onClick={confirmSegment}
          >
            Confirm segment
          </button>
        </div>
      </div>

      {/* ── Right: TM matches + glossary + AI draft ── */}
      <aside className="w-72 flex flex-col border-l border-slate-200 bg-white overflow-hidden shrink-0">
        {/* TM Matches */}
        <div className="flex flex-col flex-1 min-h-0 overflow-hidden">
          <div className="px-3 py-2 border-b border-slate-100 shrink-0">
            <h2 className="text-xs font-semibold text-slate-500 uppercase tracking-wide">
              TM Matches
            </h2>
          </div>
          <div className="flex-1 overflow-y-auto">
            {!sourceText.trim() && !isSearching ? (
              <div className="p-4 text-xs text-slate-400 space-y-2">
                <p>TM · Glossary · AI — shared across Workbench and Project.</p>
                <p>Confirmed segments are saved to the TM automatically.</p>
              </div>
            ) : (
              <MatchPanel matches={tmMatches} isLoading={isSearching} />
            )}
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
            <ul className="divide-y divide-slate-100 max-h-36 overflow-y-auto">
              {glossaryHits.map((h) => (
                <li key={h.term.id} className="px-3 py-2">
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
                  onClick={() => setTargetText(aiDraft.text)}
                >
                  Apply draft
                </button>
              </div>
            )}
          </div>
        )}
      </aside>
    </div>
  );
}
