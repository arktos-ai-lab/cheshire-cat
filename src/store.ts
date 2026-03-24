import { create } from "zustand";
import type {
  AiSuggestion,
  GlossaryHit,
  ProjectSegment,
  QaIssue,
  Settings,
  TmMatch,
  View,
} from "./types";

// ── Workbench (single-segment scratch pad) ────────────────────────────────────

interface WorkbenchState {
  sourceText: string;
  targetText: string;
  tmMatches: TmMatch[];
  glossaryHits: GlossaryHit[];
  qaIssues: QaIssue[];
  isSearching: boolean;
  aiDraft: AiSuggestion | null;
  isAiLoading: boolean;
}

// ── Project (file-based CAT workflow) ─────────────────────────────────────────

interface ProjectState {
  projectSegments: ProjectSegment[];
  activeSegmentIndex: number;
  projectFileName: string | null;
  /** Full absolute path of the original source file — used for DOCX re-export. */
  projectSourcePath: string | null;
  projectSourceLang: string | null;
  projectTargetLang: string | null;
  projectTmMatches: TmMatch[];
  projectGlossaryHits: GlossaryHit[];
  projectAiDraft: AiSuggestion | null;
  isProjectAiLoading: boolean;
  isProjectSearching: boolean;
}

// ── Combined store ────────────────────────────────────────────────────────────

interface AppState extends WorkbenchState, ProjectState {
  view: View;
  settings: Settings | null;

  // Workbench actions
  setSourceText: (text: string) => void;
  setTargetText: (text: string) => void;
  setTmMatches: (matches: TmMatch[]) => void;
  setGlossaryHits: (hits: GlossaryHit[]) => void;
  setQaIssues: (issues: QaIssue[]) => void;
  setIsSearching: (v: boolean) => void;
  applyMatch: (match: TmMatch) => void;
  setAiDraft: (draft: AiSuggestion | null) => void;
  setIsAiLoading: (v: boolean) => void;

  // Project actions
  openProject: (
    segments: ProjectSegment[],
    fileName: string,
    sourcePath: string,
    sourceLang: string | null,
    targetLang: string | null,
  ) => void;
  setActiveSegmentIndex: (i: number) => void;
  updateSegmentTarget: (index: number, target: string) => void;
  confirmSegmentAt: (index: number) => void;
  setProjectTmMatches: (matches: TmMatch[]) => void;
  setProjectGlossaryHits: (hits: GlossaryHit[]) => void;
  setProjectAiDraft: (draft: AiSuggestion | null) => void;
  setIsProjectAiLoading: (v: boolean) => void;
  setIsProjectSearching: (v: boolean) => void;

  // App actions
  setView: (view: View) => void;
  setSettings: (s: Settings) => void;
}

export const useStore = create<AppState>((set) => ({
  // Workbench initial state
  sourceText: "",
  targetText: "",
  tmMatches: [],
  glossaryHits: [],
  qaIssues: [],
  isSearching: false,
  aiDraft: null,
  isAiLoading: false,

  // Project initial state
  projectSegments: [],
  activeSegmentIndex: 0,
  projectFileName: null,
  projectSourcePath: null,
  projectSourceLang: null,
  projectTargetLang: null,
  projectTmMatches: [],
  projectGlossaryHits: [],
  projectAiDraft: null,
  isProjectAiLoading: false,
  isProjectSearching: false,

  // App initial state
  view: "workbench",
  settings: null,

  // Workbench actions
  setSourceText: (sourceText) =>
    set({ sourceText, tmMatches: [], glossaryHits: [], aiDraft: null }),
  setTargetText: (targetText) => set({ targetText }),
  setTmMatches: (tmMatches) => set({ tmMatches }),
  setGlossaryHits: (glossaryHits) => set({ glossaryHits }),
  setQaIssues: (qaIssues) => set({ qaIssues }),
  setIsSearching: (isSearching) => set({ isSearching }),
  applyMatch: (match) => set({ targetText: match.record.target }),
  setAiDraft: (aiDraft) => set({ aiDraft }),
  setIsAiLoading: (isAiLoading) => set({ isAiLoading }),

  // Project actions
  openProject: (segments, projectFileName, projectSourcePath, projectSourceLang, projectTargetLang) =>
    set({
      projectSegments: segments,
      activeSegmentIndex: 0,
      projectFileName,
      projectSourcePath,
      projectSourceLang,
      projectTargetLang,
      projectTmMatches: [],
      projectGlossaryHits: [],
      projectAiDraft: null,
    }),
  setActiveSegmentIndex: (activeSegmentIndex) =>
    set({ activeSegmentIndex, projectTmMatches: [], projectGlossaryHits: [], projectAiDraft: null }),
  updateSegmentTarget: (index, target) =>
    set((state) => {
      const segments = [...state.projectSegments];
      const seg = segments[index];
      if (!seg) return {};
      segments[index] = {
        ...seg,
        target,
        status: target.trim() ? "draft" : "untranslated",
      };
      return { projectSegments: segments };
    }),
  confirmSegmentAt: (index) =>
    set((state) => {
      const segments = [...state.projectSegments];
      const seg = segments[index];
      if (!seg || !seg.target.trim()) return {};
      segments[index] = { ...seg, status: "confirmed" };
      return { projectSegments: segments };
    }),
  setProjectTmMatches: (projectTmMatches) => set({ projectTmMatches }),
  setProjectGlossaryHits: (projectGlossaryHits) => set({ projectGlossaryHits }),
  setProjectAiDraft: (projectAiDraft) => set({ projectAiDraft }),
  setIsProjectAiLoading: (isProjectAiLoading) => set({ isProjectAiLoading }),
  setIsProjectSearching: (isProjectSearching) => set({ isProjectSearching }),

  // App actions
  setView: (view) => set({ view }),
  setSettings: (settings) => set({ settings }),
}));
