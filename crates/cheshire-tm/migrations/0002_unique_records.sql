-- Ensure at most one translation per (source, language pair) in the TM.
-- This matches Felix's original single-best-translation-per-unit model and
-- makes duplicate detection during TMX import reliable.
CREATE UNIQUE INDEX IF NOT EXISTS idx_records_unique
    ON records(source_hash, source_lang, target_lang);
