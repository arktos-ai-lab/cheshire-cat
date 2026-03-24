export type MatchType = "Exact" | "Fuzzy" | "ContextMatch";

export interface RecordMetadata {
  creator: string | null;
  client: string | null;
  domain: string | null;
  reliability: number;
  validated: boolean;
}

export interface TmRecord {
  id: string;
  source: string;
  target: string;
  source_lang: string;
  target_lang: string;
  created_at: string;
  modified_at: string;
  metadata: RecordMetadata;
}

export interface TmMatch {
  record: TmRecord;
  score: number;
  match_type: MatchType;
}

export interface GlossaryTerm {
  id: string;
  source_term: string;
  target_term: string;
  source_lang: string;
  target_lang: string;
  domain: string | null;
  note: string | null;
  forbidden: boolean;
}

export interface GlossaryHit {
  term: GlossaryTerm;
  offset: number;
}

export interface ImportStats {
  imported: number;
  skipped_duplicates: number;
  skipped_errors: number;
}

export type AiMode = "ollama" | "open_ai_compatible" | "vllm" | "deepl" | "disabled";

export interface Settings {
  sourceLang: string;
  targetLang: string;
  fuzzyThreshold: number;
  maxMatches: number;
  aiUrl: string;
  aiModel: string;
  aiMode: AiMode;
  aiApiKey: string;
  tmDbPath: string | null;
  /** Injected at runtime — true on Windows, false on macOS/Linux. */
  isWindows: boolean;
}

export interface AiSuggestion {
  text: string;
  is_fallback: boolean;
}

export interface QaIssue {
  kind: string;
  message: string;
}

export type View = "workbench" | "project" | "memory" | "glossary" | "settings" | "help";

export interface SourceUnit {
  id: string;
  source: string;
  target: string | null;
  note: string | null;
}

export interface FormatImportResult {
  units: SourceUnit[];
  source_lang: string | null;
  target_lang: string | null;
  format: string;
}

export type SegmentStatus = "untranslated" | "draft" | "confirmed";

export interface ProjectSegment {
  /** Stable ID from the source file (XLIFF trans-unit id, paragraph index, etc.) */
  id: string;
  source: string;
  target: string;
  status: SegmentStatus;
  note: string | null;
}
