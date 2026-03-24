import { useEffect, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { migrateFromFelix2, settingsGet, settingsSet } from "../api";
import type { LegacySettings } from "../api";
import type { AiMode, Settings } from "../types";

const AI_MODE_LABELS: Record<AiMode, string> = {
  ollama: "Ollama (local)",
  open_ai_compatible: "OpenAI-compatible API",
  vllm: "vLLM (local)",
  deepl: "DeepL",
  disabled: "Disabled",
};

// ── Felix 2.x migration panel ─────────────────────────────────────────────────

interface MigrationPanelProps {
  onApply: (legacy: LegacySettings) => void;
}

function MigrationPanel({ onApply }: MigrationPanelProps) {
  const [status, setStatus] = useState<
    "idle" | "checking" | "found" | "not-found" | "applied"
  >("idle");
  const [legacy, setLegacy] = useState<LegacySettings | null>(null);

  async function handleCheck() {
    setStatus("checking");
    try {
      const result = await migrateFromFelix2();
      if (result) {
        setLegacy(result);
        setStatus("found");
      } else {
        setStatus("not-found");
      }
    } catch {
      setStatus("not-found");
    }
  }

  function handleApply() {
    if (!legacy) return;
    onApply(legacy);
    setStatus("applied");
  }

  return (
    <fieldset>
      <legend className="text-xs font-semibold text-slate-500 uppercase tracking-wide mb-3">
        Migrate from Felix 2.x
      </legend>

      {status === "idle" && (
        <div className="flex items-center gap-3">
          <button className="btn-secondary py-1" onClick={handleCheck}>
            Check for Felix 2.x settings
          </button>
          <p className="text-xs text-slate-400">
            Windows only — reads the Felix 2.x registry key.
          </p>
        </div>
      )}

      {status === "checking" && (
        <p className="text-xs text-slate-400 animate-pulse">Checking registry…</p>
      )}

      {status === "not-found" && (
        <p className="text-xs text-slate-500">
          No Felix 2.x installation found on this machine.
        </p>
      )}

      {status === "found" && legacy && (
        <div className="space-y-2">
          <p className="text-xs text-slate-600 mb-2">
            Found Felix 2.x settings. Review and click Apply to import.
          </p>
          <table className="text-xs w-full">
            <tbody className="divide-y divide-slate-100">
              {legacy.source_lang && (
                <tr>
                  <td className="py-1 pr-3 text-slate-500 w-40">Source language</td>
                  <td className="py-1 font-mono text-slate-800">{legacy.source_lang}</td>
                </tr>
              )}
              {legacy.target_lang && (
                <tr>
                  <td className="py-1 pr-3 text-slate-500">Target language</td>
                  <td className="py-1 font-mono text-slate-800">{legacy.target_lang}</td>
                </tr>
              )}
              {legacy.fuzzy_threshold != null && (
                <tr>
                  <td className="py-1 pr-3 text-slate-500">Fuzzy threshold</td>
                  <td className="py-1 font-mono text-slate-800">{legacy.fuzzy_threshold}%</td>
                </tr>
              )}
              {legacy.ai_url && (
                <tr>
                  <td className="py-1 pr-3 text-slate-500">AI URL</td>
                  <td className="py-1 font-mono text-slate-800 truncate max-w-xs">{legacy.ai_url}</td>
                </tr>
              )}
              {legacy.ai_model && (
                <tr>
                  <td className="py-1 pr-3 text-slate-500">AI model</td>
                  <td className="py-1 font-mono text-slate-800">{legacy.ai_model}</td>
                </tr>
              )}
            </tbody>
          </table>
          <div className="flex gap-2 pt-1">
            <button className="btn-primary py-1" onClick={handleApply}>
              Apply to current settings
            </button>
            <button
              className="btn-secondary py-1"
              onClick={() => setStatus("idle")}
            >
              Cancel
            </button>
          </div>
        </div>
      )}

      {status === "applied" && (
        <p className="text-xs text-emerald-600 font-medium">
          Settings applied — click Save settings to persist.
        </p>
      )}
    </fieldset>
  );
}

// ── Main settings view ────────────────────────────────────────────────────────

export default function SettingsView() {
  const qc = useQueryClient();
  const [form, setForm] = useState<Settings | null>(null);
  const [saved, setSaved] = useState(false);

  const { data: settings } = useQuery({
    queryKey: ["settings"],
    queryFn: settingsGet,
  });

  useEffect(() => {
    if (settings && !form) setForm(settings);
  }, [settings, form]);

  const saveMut = useMutation({
    mutationFn: settingsSet,
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["settings"] });
      setSaved(true);
      setTimeout(() => setSaved(false), 2000);
    },
  });

  if (!form) {
    return <div className="p-8 text-slate-400 text-sm">Loading settings…</div>;
  }

  function field<K extends keyof Settings>(key: K) {
    return (value: Settings[K]) =>
      setForm((f) => f && { ...f, [key]: value });
  }

  function applyLegacy(legacy: LegacySettings) {
    setForm((f) => {
      if (!f) return f;
      return {
        ...f,
        ...(legacy.source_lang ? { sourceLang: legacy.source_lang } : {}),
        ...(legacy.target_lang ? { targetLang: legacy.target_lang } : {}),
        ...(legacy.fuzzy_threshold != null
          ? { fuzzyThreshold: legacy.fuzzy_threshold }
          : {}),
        ...(legacy.ai_url ? { aiUrl: legacy.ai_url } : {}),
        ...(legacy.ai_model ? { aiModel: legacy.ai_model } : {}),
      };
    });
  }

  const aiDisabled = form.aiMode === "disabled";
  const showApiKey = form.aiMode === "open_ai_compatible" || form.aiMode === "deepl";
  const isDeepL = form.aiMode === "deepl";
  const isVllm = form.aiMode === "vllm";

  return (
    <div className="max-w-lg p-6">
      <h1 className="text-lg font-semibold text-slate-800 mb-6">Settings</h1>

      <div className="card p-5 space-y-5">
        {/* Language pair */}
        <fieldset>
          <legend className="text-xs font-semibold text-slate-500 uppercase tracking-wide mb-3">
            Language pair
          </legend>
          <div className="grid grid-cols-2 gap-3">
            <div>
              <label className="label">Source language</label>
              <input
                className="input"
                value={form.sourceLang}
                onChange={(e) => field("sourceLang")(e.target.value)}
                placeholder=""
              />
            </div>
            <div>
              <label className="label">Target language</label>
              <input
                className="input"
                value={form.targetLang}
                onChange={(e) => field("targetLang")(e.target.value)}
                placeholder=""
              />
            </div>
          </div>
        </fieldset>

        {/* TM search */}
        <fieldset>
          <legend className="text-xs font-semibold text-slate-500 uppercase tracking-wide mb-3">
            TM search
          </legend>
          <div className="grid grid-cols-2 gap-3">
            <div>
              <label className="label">Fuzzy threshold (%)</label>
              <input
                type="number"
                min={50}
                max={100}
                className="input"
                value={form.fuzzyThreshold}
                onChange={(e) =>
                  field("fuzzyThreshold")(parseInt(e.target.value, 10))
                }
              />
            </div>
            <div>
              <label className="label">Max matches</label>
              <input
                type="number"
                min={1}
                max={20}
                className="input"
                value={form.maxMatches}
                onChange={(e) =>
                  field("maxMatches")(parseInt(e.target.value, 10))
                }
              />
            </div>
          </div>
        </fieldset>

        {/* AI suggestions */}
        <fieldset>
          <legend className="text-xs font-semibold text-slate-500 uppercase tracking-wide mb-3">
            AI suggestions
          </legend>

          <div className="space-y-3">
            <div>
              <label className="label">AI backend</label>
              <select
                className="input"
                value={form.aiMode}
                onChange={(e) => field("aiMode")(e.target.value as AiMode)}
              >
                {(Object.keys(AI_MODE_LABELS) as AiMode[]).map((mode) => (
                  <option key={mode} value={mode}>
                    {AI_MODE_LABELS[mode]}
                  </option>
                ))}
              </select>
            </div>

            {!aiDisabled && (
              <>
                {!isDeepL && (
                  <div>
                    <label className="label">
                      {form.aiMode === "ollama" ? "Ollama URL" : "API base URL"}
                    </label>
                    <input
                      className="input font-mono text-xs"
                      value={form.aiUrl}
                      onChange={(e) => field("aiUrl")(e.target.value)}
                      placeholder={
                        form.aiMode === "ollama"
                          ? "http://localhost:11434"
                          : form.aiMode === "vllm"
                          ? "http://localhost:8000"
                          : "https://api.openai.com"
                      }
                    />
                  </div>
                )}

                {!isDeepL && (
                  <div>
                    <label className="label">Model</label>
                    <input
                      className="input"
                      value={form.aiModel}
                      onChange={(e) => field("aiModel")(e.target.value)}
                      placeholder={
                        form.aiMode === "ollama"
                          ? "llama3.2:3b"
                          : isVllm
                          ? "Qwen/Qwen2.5-Coder-7B-Instruct"
                          : "gpt-4o-mini"
                      }
                    />
                    {form.aiMode === "ollama" && (
                      <p className="mt-1 text-xs text-slate-400">
                        Recommended for corporate laptops: llama3.2:3b,
                        phi4-mini:3.8b, gemma3:4b. Max ~4 GB RAM.
                      </p>
                    )}
                    {isVllm && (
                      <p className="mt-1 text-xs text-slate-400">
                        Use the full HuggingFace model ID (e.g.
                        Qwen/Qwen2.5-7B-Instruct). No API key required for
                        local servers.
                      </p>
                    )}
                  </div>
                )}

                {showApiKey && (
                  <div>
                    <label className="label">
                      {isDeepL ? "DeepL API key" : "API key"}
                    </label>
                    <input
                      type="password"
                      className="input font-mono text-xs"
                      value={form.aiApiKey}
                      onChange={(e) => field("aiApiKey")(e.target.value)}
                      placeholder={
                        isDeepL
                          ? "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx:fx"
                          : "sk-…"
                      }
                      autoComplete="off"
                    />
                    {isDeepL && (
                      <p className="mt-1 text-xs text-slate-400">
                        Free tier: api-free.deepl.com · Pro tier: api.deepl.com.
                        Get your key at deepl.com/account/summary.
                      </p>
                    )}
                  </div>
                )}
              </>
            )}
          </div>
        </fieldset>

        {/* Database path */}
        <fieldset>
          <legend className="text-xs font-semibold text-slate-500 uppercase tracking-wide mb-3">
            Database
          </legend>
          <div>
            <label className="label">
              TM database path (leave blank for default)
            </label>
            <input
              className="input font-mono text-xs"
              value={form.tmDbPath ?? ""}
              onChange={(e) => field("tmDbPath")(e.target.value || null)}
              placeholder="Default: <AppData>/tm.db"
            />
          </div>
        </fieldset>

        {/* Felix 2.x migration — Windows only */}
        {form.isWindows && <MigrationPanel onApply={applyLegacy} />}
      </div>

      <div className="flex items-center gap-3 mt-4">
        <button
          className="btn-primary"
          disabled={saveMut.isPending}
          onClick={() => saveMut.mutate(form)}
        >
          {saveMut.isPending ? "Saving…" : "Save settings"}
        </button>
        {saved && (
          <span className="text-sm text-emerald-600 font-medium">Saved!</span>
        )}
        {saveMut.isError && (
          <span className="text-sm text-red-600">{String(saveMut.error)}</span>
        )}
      </div>
    </div>
  );
}
