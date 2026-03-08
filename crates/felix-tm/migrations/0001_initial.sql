-- Felix TM initial schema

CREATE TABLE IF NOT EXISTS records (
    id          TEXT PRIMARY KEY,
    source      TEXT NOT NULL,
    target      TEXT NOT NULL,
    source_lang TEXT NOT NULL,
    target_lang TEXT NOT NULL,
    -- sha256(normalised source) for fast exact-match lookup
    source_hash TEXT NOT NULL,
    created_at  TEXT NOT NULL,
    modified_at TEXT NOT NULL,
    creator     TEXT,
    client      TEXT,
    domain      TEXT,
    reliability INTEGER NOT NULL DEFAULT 100,
    validated   INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_lang_pair   ON records(source_lang, target_lang);
CREATE INDEX IF NOT EXISTS idx_source_hash ON records(source_hash);
CREATE INDEX IF NOT EXISTS idx_modified    ON records(modified_at);

CREATE TABLE IF NOT EXISTS glossary_terms (
    id          TEXT PRIMARY KEY,
    source_term TEXT NOT NULL,
    target_term TEXT NOT NULL,
    source_lang TEXT NOT NULL,
    target_lang TEXT NOT NULL,
    domain      TEXT,
    note        TEXT,
    -- 1 = this term is forbidden (do not use)
    forbidden   INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_gloss_lang ON glossary_terms(source_lang, target_lang);
CREATE INDEX IF NOT EXISTS idx_gloss_src  ON glossary_terms(source_term);

-- Projects track which file is being translated and its current state
CREATE TABLE IF NOT EXISTS projects (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    source_lang TEXT NOT NULL,
    target_lang TEXT NOT NULL,
    source_file TEXT,
    created_at  TEXT NOT NULL
);

-- Individual segments within a project
CREATE TABLE IF NOT EXISTS segments (
    id          TEXT PRIMARY KEY,
    project_id  TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    seq         INTEGER NOT NULL,       -- ordering within project
    source      TEXT NOT NULL,
    target      TEXT,
    -- untranslated | draft | translated | reviewed | approved
    status      TEXT NOT NULL DEFAULT 'untranslated',
    match_score REAL,
    match_type  TEXT,                   -- exact | fuzzy | context | mt | ai | manual
    locked      INTEGER NOT NULL DEFAULT 0,
    UNIQUE(project_id, seq)
);

CREATE INDEX IF NOT EXISTS idx_seg_project ON segments(project_id, seq);
CREATE INDEX IF NOT EXISTS idx_seg_status  ON segments(status);

-- Full edit history per segment for auditability
CREATE TABLE IF NOT EXISTS segment_history (
    id          TEXT PRIMARY KEY,
    segment_id  TEXT NOT NULL REFERENCES segments(id) ON DELETE CASCADE,
    target      TEXT NOT NULL,
    changed_at  TEXT NOT NULL,
    changed_by  TEXT,
    source      TEXT NOT NULL           -- 'manual' | 'tm' | 'mt' | 'ai'
);

CREATE INDEX IF NOT EXISTS idx_hist_segment ON segment_history(segment_id, changed_at);
