import { invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";

import type {
  AiSuggestion,
  FormatImportResult,
  GlossaryHit,
  GlossaryTerm,
  ImportStats,
  QaIssue,
  Settings,
  SourceUnit,
  TmMatch,
  TmRecord,
} from "../types";

// ── TM commands ───────────────────────────────────────────────────────────────

export function tmSearch(query: string): Promise<TmMatch[]> {
  return invoke("tm_search", { query });
}

export function tmExact(source: string): Promise<TmRecord[]> {
  return invoke("tm_exact", { source });
}

export function tmImport(path: string): Promise<ImportStats> {
  return invoke("tm_import", { path });
}

export function tmExport(path: string): Promise<void> {
  return invoke("tm_export", { path });
}

export function tmAddRecord(source: string, target: string): Promise<string> {
  return invoke("tm_add_record", { source, target });
}

export function tmDeleteRecord(id: string): Promise<void> {
  return invoke("tm_delete_record", { id });
}

export function tmCount(): Promise<number> {
  return invoke("tm_count");
}

export function tmList(limit: number, offset: number): Promise<TmRecord[]> {
  return invoke("tm_list", { limit, offset });
}

// ── Glossary commands ─────────────────────────────────────────────────────────

export function glossaryLookup(text: string): Promise<GlossaryHit[]> {
  return invoke("glossary_lookup", { text });
}

export function glossaryInsert(params: {
  sourceTerm: string;
  targetTerm: string;
  domain?: string;
  note?: string;
  forbidden: boolean;
}): Promise<string> {
  return invoke("glossary_insert", {
    sourceTerm: params.sourceTerm,
    targetTerm: params.targetTerm,
    domain: params.domain ?? null,
    note: params.note ?? null,
    forbidden: params.forbidden,
  });
}

export function glossaryDelete(id: string): Promise<void> {
  return invoke("glossary_delete", { id });
}

export function glossaryListAll(): Promise<GlossaryTerm[]> {
  return invoke("glossary_list_all");
}

export function glossaryCount(): Promise<number> {
  return invoke("glossary_count");
}

// ── Settings commands ─────────────────────────────────────────────────────────

export function settingsGet(): Promise<Settings> {
  return invoke("settings_get");
}

export function settingsSet(newSettings: Settings): Promise<void> {
  return invoke("settings_set", { newSettings });
}

// ── QA commands ───────────────────────────────────────────────────────────────

export function qaRun(source: string, target: string): Promise<QaIssue[]> {
  return invoke("qa_run", { source, target });
}

// ── Dialog helpers ────────────────────────────────────────────────────────────

export async function openTmxFile(): Promise<string | null> {
  const selected = await open({
    multiple: false,
    filters: [{ name: "Translation Memory", extensions: ["tmx", "xliff", "xlf"] }],
  });
  return typeof selected === "string" ? selected : null;
}

export async function saveTmxFile(): Promise<string | null> {
  const path = await save({
    filters: [{ name: "Translation Memory", extensions: ["tmx"] }],
    defaultPath: "export.tmx",
  });
  return path ?? null;
}

// ── Format commands ───────────────────────────────────────────────────────────

export function formatImport(path: string): Promise<FormatImportResult> {
  return invoke("format_import", { path });
}

export function formatExportXliff(
  path: string,
  units: SourceUnit[],
  sourceLang: string,
  targetLang: string,
): Promise<void> {
  return invoke("format_export_xliff", { path, units, sourceLang, targetLang });
}

export function formatExportPo(path: string, units: SourceUnit[]): Promise<void> {
  return invoke("format_export_po", { path, units });
}

export function formatExportJson(path: string, units: SourceUnit[]): Promise<void> {
  return invoke("format_export_json", { path, units });
}

export function formatExportCsv(path: string, units: SourceUnit[]): Promise<void> {
  return invoke("format_export_csv", { path, units });
}

export function formatExportDocx(
  sourcePath: string,
  destPath: string,
  translations: Record<string, string>,
): Promise<void> {
  return invoke("format_export_docx", { sourcePath, destPath, translations });
}

export async function openSourceFile(): Promise<string | null> {
  const selected = await open({
    multiple: false,
    filters: [
      {
        name: "Translatable files",
        extensions: ["docx", "xlsx", "xliff", "xlf", "html", "htm", "txt", "po", "json", "csv"],
      },
    ],
  });
  return typeof selected === "string" ? selected : null;
}

export async function saveXliffFile(baseName?: string): Promise<string | null> {
  const path = await save({
    filters: [{ name: "XLIFF", extensions: ["xliff"] }],
    defaultPath: baseName ? `${baseName}.xliff` : "export.xliff",
  });
  return path ?? null;
}

export async function savePoFile(baseName?: string): Promise<string | null> {
  const path = await save({
    filters: [{ name: "PO (Gettext)", extensions: ["po"] }],
    defaultPath: baseName ? `${baseName}.po` : "export.po",
  });
  return path ?? null;
}

export async function saveJsonFile(baseName?: string): Promise<string | null> {
  const path = await save({
    filters: [{ name: "JSON", extensions: ["json"] }],
    defaultPath: baseName ? `${baseName}.json` : "export.json",
  });
  return path ?? null;
}

export async function saveCsvFile(baseName?: string): Promise<string | null> {
  const path = await save({
    filters: [{ name: "CSV", extensions: ["csv"] }],
    defaultPath: baseName ? `${baseName}.csv` : "export.csv",
  });
  return path ?? null;
}

export async function saveDocxFile(baseName?: string): Promise<string | null> {
  const path = await save({
    filters: [{ name: "Word Document", extensions: ["docx"] }],
    defaultPath: baseName ? `${baseName}_translated.docx` : "translated.docx",
  });
  return path ?? null;
}

export async function saveBilingualDocxFile(baseName?: string): Promise<string | null> {
  const path = await save({
    filters: [{ name: "Word Document", extensions: ["docx"] }],
    defaultPath: baseName ? `${baseName}_bilingual.docx` : "bilingual.docx",
  });
  return path ?? null;
}

export function formatExportBilingualDocx(
  path: string,
  units: SourceUnit[],
  sourceLang: string,
  targetLang: string,
): Promise<void> {
  return invoke("format_export_bilingual_docx", { path, units, sourceLang, targetLang });
}

// ── AI commands ───────────────────────────────────────────────────────────────

export interface TmHit {
  source: string;
  target: string;
  score: number;
}

export interface GlossaryHitAi {
  source_term: string;
  target_term: string;
}

export function aiSuggest(params: {
  source: string;
  sourceLang: string;
  targetLang: string;
  tmMatches: TmHit[];
  glossaryHits: GlossaryHitAi[];
  domain?: string | null;
  prevTarget?: string | null;
}): Promise<AiSuggestion | null> {
  return invoke("ai_suggest", {
    source: params.source,
    sourceLang: params.sourceLang,
    targetLang: params.targetLang,
    tmMatches: params.tmMatches,
    glossaryHits: params.glossaryHits,
    domain: params.domain ?? null,
    prevTarget: params.prevTarget ?? null,
  });
}

// ── Office COM bridge ─────────────────────────────────────────────────────────

export function officeGetWordSelection(): Promise<string | null> {
  return invoke("office_get_word_selection");
}
export function officeInsertIntoWord(text: string): Promise<void> {
  return invoke("office_insert_into_word", { text });
}
export function officeWordIsRunning(): Promise<boolean> {
  return invoke("office_word_is_running");
}

export function officeGetExcelSelection(): Promise<string | null> {
  return invoke("office_get_excel_selection");
}
export function officeInsertIntoExcel(text: string): Promise<void> {
  return invoke("office_insert_into_excel", { text });
}
export function officeExcelIsRunning(): Promise<boolean> {
  return invoke("office_excel_is_running");
}

export function officeGetPptSelection(): Promise<string | null> {
  return invoke("office_get_ppt_selection");
}
export function officeInsertIntoPpt(text: string): Promise<void> {
  return invoke("office_insert_into_ppt", { text });
}
export function officePptIsRunning(): Promise<boolean> {
  return invoke("office_ppt_is_running");
}

// ── Registry migration ────────────────────────────────────────────────────────

export interface LegacySettings {
  source_lang: string | null;
  target_lang: string | null;
  fuzzy_threshold: number | null;
  ai_url: string | null;
  ai_model: string | null;
}

export function migrateFromFelix2(): Promise<LegacySettings | null> {
  return invoke("migrate_from_felix2");
}

// ── Update check ──────────────────────────────────────────────────────────────

export interface UpdateInfo {
  available: boolean;
  current_version: string;
  latest_version: string;
  release_url: string;
  release_notes: string;
}

export function checkForUpdate(): Promise<UpdateInfo> {
  return invoke("check_for_update");
}
