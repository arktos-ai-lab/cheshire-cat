import { useState } from "react";

function Section({ title, children }: { title: string; children: React.ReactNode }) {
  const [open, setOpen] = useState(false);
  return (
    <div className="border border-slate-200 rounded-lg overflow-hidden">
      <button
        className="w-full flex items-center justify-between px-4 py-3 text-left text-sm font-semibold text-slate-700 bg-white hover:bg-slate-50 transition-colors"
        onClick={() => setOpen((o) => !o)}
      >
        {title}
        <span className="text-slate-400 text-xs ml-2 shrink-0">{open ? "▲" : "▼"}</span>
      </button>
      {open && (
        <div className="px-4 py-4 text-sm text-slate-600 bg-slate-50 border-t border-slate-200 space-y-4">
          {children}
        </div>
      )}
    </div>
  );
}

function Tip({ children }: { children: React.ReactNode }) {
  return (
    <div className="flex gap-2 bg-blue-50 border border-blue-200 rounded p-3 text-blue-800 text-xs leading-relaxed">
      <span className="font-bold shrink-0">Tip</span>
      <span>{children}</span>
    </div>
  );
}

function Note({ children }: { children: React.ReactNode }) {
  return (
    <div className="flex gap-2 bg-amber-50 border border-amber-200 rounded p-3 text-amber-800 text-xs leading-relaxed">
      <span className="font-bold shrink-0">Note</span>
      <span>{children}</span>
    </div>
  );
}

function Kbd({ children }: { children: React.ReactNode }) {
  return (
    <kbd className="bg-slate-200 border border-slate-300 rounded px-1.5 py-0.5 font-mono text-xs text-slate-700">
      {children}
    </kbd>
  );
}

function H2({ children }: { children: React.ReactNode }) {
  return <h3 className="font-semibold text-slate-700 text-sm mt-3 first:mt-0">{children}</h3>;
}

function Steps({ items }: { items: string[] }) {
  return (
    <ol className="list-decimal pl-5 space-y-1.5">
      {items.map((item, i) => (
        <li key={i} dangerouslySetInnerHTML={{ __html: item }} />
      ))}
    </ol>
  );
}

function Bullets({ items }: { items: React.ReactNode[] }) {
  return (
    <ul className="list-disc pl-5 space-y-1.5">
      {items.map((item, i) => <li key={i}>{item}</li>)}
    </ul>
  );
}

export default function HelpView() {
  return (
    <div className="h-full overflow-y-auto">
      <div className="max-w-2xl p-6 space-y-3">

        <div className="mb-5">
          <h1 className="text-lg font-semibold text-slate-800 mb-1">Cheshire CAT — Help</h1>
          <p className="text-sm text-slate-500">
            Felix is a computer-assisted translation tool. It assists you in producing better
            translations more quickly while keeping you firmly in control. All features work
            fully offline; an internet connection is only needed for cloud AI backends.
          </p>
        </div>

        {/* ── OVERVIEW ── */}
        <Section title="1. Overview — How the Four Parts Fit Together">
          {/* Architecture diagram */}
          <div className="rounded-lg border border-slate-200 overflow-hidden text-xs">
            {/* Shared layer */}
            <div className="bg-slate-700 text-white text-center py-1.5 font-semibold tracking-wide text-xs">
              SHARED RESOURCES — persist across sessions
            </div>
            <div className="grid grid-cols-2 divide-x divide-slate-200 bg-white">
              <div className="p-3 space-y-1">
                <div className="font-semibold text-slate-700">Translation Memory (TM)</div>
                <div className="text-slate-500">SQLite database of confirmed source–target pairs. Grows automatically. Queried for every segment you translate.</div>
              </div>
              <div className="p-3 space-y-1">
                <div className="font-semibold text-slate-700">Glossary</div>
                <div className="text-slate-500">Database of approved terms and forbidden words. Queried alongside the TM. Managed in the Glossary view.</div>
              </div>
            </div>
            {/* Arrow row */}
            <div className="grid grid-cols-2 divide-x divide-slate-200 bg-slate-50 text-center py-1 text-slate-400 font-mono text-base">
              <div>↕ reads &amp; writes</div>
              <div>↕ reads &amp; writes</div>
            </div>
            {/* Interface layer */}
            <div className="bg-slate-100 text-slate-600 text-center py-1.5 font-semibold tracking-wide text-xs border-t border-slate-200">
              TRANSLATION INTERFACES — two ways to access the same resources
            </div>
            <div className="grid grid-cols-2 divide-x divide-slate-200 bg-white">
              <div className="p-3 space-y-1">
                <div className="font-semibold text-slate-700">Workbench</div>
                <div className="text-slate-500">One segment at a time. Type or paste source text, get TM + glossary + AI suggestions, confirm to save back to the TM.</div>
              </div>
              <div className="p-3 space-y-1">
                <div className="font-semibold text-slate-700">Project</div>
                <div className="text-slate-500">Whole file at once. Open a document, work through all segments in a table, export the finished translation.</div>
              </div>
            </div>
          </div>

          <H2>The key insight</H2>
          <p>
            The <strong>TM and Glossary are databases</strong> — they are not tied to any
            particular file or session. Both Workbench and Project read from and write to the
            same TM and Glossary. A segment confirmed in Workbench is immediately available as a
            TM match in Project, and vice versa. Your TM grows continuously across all your work.
          </p>

          <H2>What happens when you confirm a segment</H2>
          <ol className="list-decimal pl-5 space-y-1.5">
            <li>You type or paste a <strong>source</strong> segment.</li>
            <li>Felix queries the <strong>TM</strong> — if similar source text was confirmed before, the stored translation appears as a match with a % score.</li>
            <li>Felix queries the <strong>Glossary</strong> — any technical terms in the source appear as hits alongside the TM results.</li>
            <li>If AI is configured, a <strong>draft translation</strong> is generated, informed by the TM matches and glossary hits.</li>
            <li>You edit the target text (using the match, draft, or typing from scratch).</li>
            <li>You <strong>confirm</strong> — the source + target pair is saved to the TM. Next time you see a similar sentence, Felix will suggest this translation.</li>
          </ol>

          <H2>When to use Workbench vs. Project</H2>
          <table className="w-full text-xs border-collapse">
            <thead>
              <tr className="text-left bg-slate-100">
                <th className="py-1.5 px-2 border border-slate-200">Use Workbench when…</th>
                <th className="py-1.5 px-2 border border-slate-200">Use Project when…</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-100">
              <tr>
                <td className="py-1 px-2 border border-slate-200">You are translating directly in Word or another editor and want Felix alongside for TM look-ups</td>
                <td className="py-1 px-2 border border-slate-200">You have received a file to translate (DOCX, XLIFF, PO, etc.) and want to work through every segment systematically</td>
              </tr>
              <tr>
                <td className="py-1 px-2 border border-slate-200">You want to look up a single term or sentence quickly</td>
                <td className="py-1 px-2 border border-slate-200">You need to track progress (how many segments are done) and export a finished file</td>
              </tr>
              <tr>
                <td className="py-1 px-2 border border-slate-200">You want to manually add a source–target pair to the TM</td>
                <td className="py-1 px-2 border border-slate-200">You are working with a structured format that must be exported (XLIFF handoff, PO for software, etc.)</td>
              </tr>
            </tbody>
          </table>

          <Note>
            The TM and Glossary are entirely separate from any open Project file. Closing or
            discarding a Project does not affect the TM — all confirmed segments stay in the
            database permanently unless you explicitly delete them from the Memory Browser.
          </Note>
        </Section>

        {/* ── WORKBENCH ── */}
        <Section title="2. Workbench — Single-Segment Translation">
          <H2>Basic translation cycle</H2>
          <Steps items={[
            "Type or paste a source sentence into the <strong>Source</strong> field. Felix immediately searches the TM and glossary.",
            "Review <strong>TM matches</strong> on the right panel. Each match shows a percentage score, the stored source, and the stored translation. Click any match to copy its translation into the Target field.",
            "If an AI draft appears, click <strong>Apply draft</strong> to copy it into the Target field.",
            "Edit the Target field until the translation is correct.",
            "Click <strong>Confirm segment</strong> (or press <kbd>Ctrl+Enter</kbd>) to save the pair to the TM.",
          ]} />
          <Tip>
            If you are translating a Word document, use <strong>Get from Word</strong> to copy the
            currently selected text from Word into the Source field, and <strong>Insert into
            Word</strong> to paste the confirmed translation back. This avoids switching focus
            between applications.
          </Tip>
          <H2>TM match scores</H2>
          <p>
            Scores range from 0 % to 100 %. A 100 % match means the source is identical to a
            stored entry. Scores below the <strong>fuzzy threshold</strong> (configured in
            Settings, default 70 %) are not shown.
          </p>
          <Note>
            The active TM is shared between Workbench and Project — confirmed segments in either
            view are immediately available in the other.
          </Note>
          <H2>Glossary hits</H2>
          <p>
            Glossary matches appear below TM results. Each hit shows the source term, its
            translation, and an optional domain/note. If a term is marked <strong>Forbidden</strong>,
            a warning appears in QA if that term is found in your target text.
          </p>
        </Section>

        {/* ── PROJECT ── */}
        <Section title="3. Project — File-Based Translation">
          <H2>Opening a file</H2>
          <Steps items={[
            "Click <strong>Open file</strong> in the Project toolbar.",
            "Select a supported file: DOCX, XLIFF (.xlf / .xliff), PO, HTML, plain text, JSON, CSV, or XLSX.",
            "Felix segments the file and shows every translation unit as a row in the segment table. Each row shows its ID, source text, current translation (if any), and status.",
          ]} />
          <H2>Translating segments</H2>
          <Steps items={[
            "Click a row to open it in the editor pane. TM matches, glossary hits, and the AI draft appear on the right.",
            "Click a TM match to load it into the target editor, or click <strong>Apply draft</strong> for the AI suggestion.",
            "Edit the target text as needed.",
            "Press <kbd>Ctrl+Enter</kbd> to confirm the segment. The status changes to <em>Translated</em> and focus moves to the next untranslated segment automatically.",
            "Use <kbd>↑</kbd> / <kbd>↓</kbd> to navigate between rows without confirming.",
          ]} />
          <Tip>
            Felix auto-advances to the next <em>untranslated</em> segment when you confirm.
            To return to a translated segment, click its row or use the arrow keys.
          </Tip>
          <H2>Segment statuses</H2>
          <table className="w-full text-xs border-collapse">
            <thead>
              <tr className="text-left bg-slate-100">
                <th className="py-1.5 px-2 border border-slate-200">Status</th>
                <th className="py-1.5 px-2 border border-slate-200">Meaning</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-100">
              <tr><td className="py-1 px-2 border border-slate-200">Untranslated</td><td className="py-1 px-2 border border-slate-200">No target text yet</td></tr>
              <tr><td className="py-1 px-2 border border-slate-200">Translated</td><td className="py-1 px-2 border border-slate-200">Confirmed by the user</td></tr>
            </tbody>
          </table>
          <H2>Exporting</H2>
          <p>Use the export buttons in the toolbar to save your work:</p>
          <table className="w-full text-xs border-collapse">
            <thead>
              <tr className="text-left bg-slate-100">
                <th className="py-1.5 px-2 border border-slate-200">Button</th>
                <th className="py-1.5 px-2 border border-slate-200">Output</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-100">
              <tr><td className="py-1 px-2 border border-slate-200 font-mono">XLIFF</td><td className="py-1 px-2 border border-slate-200">Bilingual XLIFF 1.2 file — industry-standard exchange format</td></tr>
              <tr><td className="py-1 px-2 border border-slate-200 font-mono">PO</td><td className="py-1 px-2 border border-slate-200">GNU gettext PO file for software localisation</td></tr>
              <tr><td className="py-1 px-2 border border-slate-200 font-mono">JSON</td><td className="py-1 px-2 border border-slate-200">Key–value JSON file for web/app localisation</td></tr>
              <tr><td className="py-1 px-2 border border-slate-200 font-mono">CSV</td><td className="py-1 px-2 border border-slate-200">Spreadsheet-compatible CSV with id, source, target, note columns</td></tr>
              <tr><td className="py-1 px-2 border border-slate-200 font-mono">DOCX</td><td className="py-1 px-2 border border-slate-200">Translated Word document — target text substituted into original layout</td></tr>
            </tbody>
          </table>
          <Note>
            DOCX export rebuilds the original document with translations replacing the source
            text. Formatting (bold, italic, font size) is preserved. Export is only available
            when the original file was a DOCX.
          </Note>
        </Section>

        {/* ── TM ── */}
        <Section title="4. Translation Memory (TM)">
          <p>
            The TM stores confirmed source–target pairs in a local <strong>SQLite database</strong>.
            It grows automatically as you confirm segments in Workbench or Project.
          </p>
          <H2>TM record metadata</H2>
          <p>
            Each record stores the following fields in addition to the source and target text:
          </p>
          <table className="w-full text-xs border-collapse">
            <thead>
              <tr className="text-left bg-slate-100">
                <th className="py-1.5 px-2 border border-slate-200">Field</th>
                <th className="py-1.5 px-2 border border-slate-200">Description</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-100">
              <tr><td className="py-1 px-2 border border-slate-200">Source language</td><td className="py-1 px-2 border border-slate-200">BCP-47 code (e.g. <code>en</code>, <code>fr</code>)</td></tr>
              <tr><td className="py-1 px-2 border border-slate-200">Target language</td><td className="py-1 px-2 border border-slate-200">BCP-47 code (e.g. <code>ja</code>, <code>zh-hans</code>)</td></tr>
              <tr><td className="py-1 px-2 border border-slate-200">Creator</td><td className="py-1 px-2 border border-slate-200">Username of the translator who added the record</td></tr>
              <tr><td className="py-1 px-2 border border-slate-200">Client</td><td className="py-1 px-2 border border-slate-200">Client name — imported from TMX metadata</td></tr>
              <tr><td className="py-1 px-2 border border-slate-200">Domain</td><td className="py-1 px-2 border border-slate-200">Subject field (e.g. Legal, Medical, IT)</td></tr>
              <tr><td className="py-1 px-2 border border-slate-200">Reliability</td><td className="py-1 px-2 border border-slate-200">Integer quality score (1–10) — set on import or manually</td></tr>
              <tr><td className="py-1 px-2 border border-slate-200">Validated</td><td className="py-1 px-2 border border-slate-200">Boolean — marks records that have passed review</td></tr>
              <tr><td className="py-1 px-2 border border-slate-200">Created</td><td className="py-1 px-2 border border-slate-200">Timestamp of creation</td></tr>
              <tr><td className="py-1 px-2 border border-slate-200">Modified</td><td className="py-1 px-2 border border-slate-200">Timestamp of last modification</td></tr>
            </tbody>
          </table>
          <H2>Importing TM data</H2>
          <Steps items={[
            "In the Memory Browser, click <strong>Import TM</strong>.",
            "Select a <strong>TMX</strong> or <strong>XLIFF</strong> file. TMX is the industry-standard TM exchange format exported by most CAT tools (SDL Trados, memoQ, OmegaT, etc.).",
            "Felix reads all translation units and stores them in the database. Metadata fields (creator, client, domain, reliability, validated) are imported from TMX attributes where present.",
          ]} />
          <Tip>
            To build a TM from a bilingual CSV, import the file in the Project view, confirm all
            segments with <Kbd>Ctrl+Enter</Kbd>, then your translations are in the TM automatically.
          </Tip>
          <H2>Exporting the TM</H2>
          <p>
            Click <strong>Export TMX</strong> in the Memory Browser to save the full TM as a
            standard TMX file. Use this for backup, sharing with a client, or importing into
            another CAT tool.
          </p>
          <H2>Fuzzy matching</H2>
          <p>
            Felix uses a normalised edit-distance algorithm to score each stored source against
            the current query. You control which matches appear via two settings:
          </p>
          <Bullets items={[
            <><strong>Fuzzy threshold</strong> (Settings): minimum score for a match to be shown. Raise this for stricter matches; lower it to see more suggestions.</>,
            <><strong>Max matches</strong> (Settings): maximum number of matches displayed per segment. Defaults to 5.</>,
          ]} />
          <Note>
            A 100 % match means the source text is character-for-character identical to a stored
            entry. Near-100 % matches often differ only in punctuation, capitalisation, or a
            single word — review them carefully before accepting.
          </Note>
        </Section>

        {/* ── MEMORY BROWSER ── */}
        <Section title="5. Memory Browser">
          <p>
            The Memory Browser gives you a full view of the TM database. Use it to inspect,
            search, edit, or delete individual records.
          </p>
          <H2>Searching the TM</H2>
          <p>
            Type any text in the search box to filter records whose source or target contains
            that string. Results update as you type. This is equivalent to a concordance search —
            useful when you need to see how a particular term was handled in past translations.
          </p>
          <H2>Editing and deleting records</H2>
          <Bullets items={[
            "Click a record to select it.",
            "Edit the source or target text directly in the record row.",
            <>Click <strong>Delete</strong> to permanently remove a record from the TM. This cannot be undone — export a TMX backup first if in doubt.</>,
          ]} />
          <Tip>
            Use the Memory Browser to perform a find-and-replace style cleanup. Search for a
            deprecated product name, then edit each matching record to use the new name.
          </Tip>
        </Section>

        {/* ── GLOSSARY ── */}
        <Section title="6. Glossary">
          <p>
            A glossary ensures consistent use of domain-specific terminology. Glossary hits
            appear automatically alongside TM matches in both Workbench and Project as you
            translate.
          </p>
          <H2>Adding terms</H2>
          <Steps items={[
            "Go to the <strong>Glossary</strong> view.",
            "Click <strong>Add term</strong>.",
            "Enter the source term, its translation, and optionally a domain and a note.",
            "Check <strong>Forbidden</strong> if this term must not appear in the target (see below).",
            "Click <strong>Save</strong>.",
          ]} />
          <H2>Glossary term fields</H2>
          <table className="w-full text-xs border-collapse">
            <thead>
              <tr className="text-left bg-slate-100">
                <th className="py-1.5 px-2 border border-slate-200">Field</th>
                <th className="py-1.5 px-2 border border-slate-200">Description</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-100">
              <tr><td className="py-1 px-2 border border-slate-200">Source</td><td className="py-1 px-2 border border-slate-200">The term in the source language</td></tr>
              <tr><td className="py-1 px-2 border border-slate-200">Target</td><td className="py-1 px-2 border border-slate-200">The approved translation of the term</td></tr>
              <tr><td className="py-1 px-2 border border-slate-200">Domain</td><td className="py-1 px-2 border border-slate-200">Subject field (e.g. Legal, Medical, IT) — optional</td></tr>
              <tr><td className="py-1 px-2 border border-slate-200">Note</td><td className="py-1 px-2 border border-slate-200">Free-text usage note or context — optional</td></tr>
              <tr><td className="py-1 px-2 border border-slate-200">Forbidden</td><td className="py-1 px-2 border border-slate-200">If checked, QA will flag this term if it appears in the target</td></tr>
            </tbody>
          </table>
          <H2>Forbidden terms</H2>
          <p>
            Mark a glossary entry as <strong>Forbidden</strong> to prevent its use in translated
            text. Common uses:
          </p>
          <Bullets items={[
            "Deprecated product names that must no longer appear in translations",
            "Words banned by a client style guide",
            "Informal or offensive terms that should never be used in formal communications",
          ]} />
          <p>
            When QA detects a forbidden term in your target text, it reports an issue with the
            offending term highlighted.
          </p>
          <Note>
            Glossary matching is case-insensitive and checks for whole-word occurrences in the
            source segment. Very short terms (single characters) may produce false positives —
            use the Domain and Note fields to add context.
          </Note>
        </Section>

        {/* ── AI ── */}
        <Section title="7. AI Suggestions">
          <p>
            Felix can connect to an AI backend to generate a draft translation for the current
            segment. The prompt includes TM matches and glossary hits as context, so the AI is
            aware of your preferred terminology and past translations.
          </p>
          <H2>Configuring the AI backend</H2>
          <p>
            Go to <strong>Settings → AI suggestions</strong> and select a backend:
          </p>
          <table className="w-full text-xs border-collapse">
            <thead>
              <tr className="text-left bg-slate-100">
                <th className="py-1.5 px-2 border border-slate-200">Backend</th>
                <th className="py-1.5 px-2 border border-slate-200">Description</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-100">
              <tr>
                <td className="py-1 px-2 border border-slate-200 font-medium">Ollama</td>
                <td className="py-1 px-2 border border-slate-200">
                  Local inference via <code>ollama serve</code>. No data leaves your machine.
                  Default URL: <code>http://localhost:11434</code>.
                  Recommended models: <code>llama3.2:3b</code>, <code>phi4-mini:3.8b</code>, <code>gemma3:4b</code>.
                </td>
              </tr>
              <tr>
                <td className="py-1 px-2 border border-slate-200 font-medium">vLLM</td>
                <td className="py-1 px-2 border border-slate-200">
                  Local OpenAI-compatible server (e.g. <code>vllm serve …</code>).
                  Default URL: <code>http://localhost:8000</code>.
                  Enter the full HuggingFace model ID (e.g. <code>Qwen/Qwen2.5-7B-Instruct</code>).
                  No API key required.
                </td>
              </tr>
              <tr>
                <td className="py-1 px-2 border border-slate-200 font-medium">OpenAI-compatible</td>
                <td className="py-1 px-2 border border-slate-200">
                  Any hosted API that follows the <code>/v1/chat/completions</code> format
                  (OpenAI, Mistral, Groq, Anthropic-compatible proxies, etc.).
                  Requires a base URL and API key.
                </td>
              </tr>
              <tr>
                <td className="py-1 px-2 border border-slate-200 font-medium">DeepL</td>
                <td className="py-1 px-2 border border-slate-200">
                  Machine translation via the DeepL API. Always invoked regardless of TM score.
                  Requires a DeepL API key (free tier available).
                </td>
              </tr>
              <tr>
                <td className="py-1 px-2 border border-slate-200 font-medium">Disabled</td>
                <td className="py-1 px-2 border border-slate-200">
                  Hides the AI panel entirely. All other features continue to work.
                </td>
              </tr>
            </tbody>
          </table>
          <H2>Using AI drafts</H2>
          <Steps items={[
            "Look up a segment. If the AI backend is reachable, a draft appears in the AI panel.",
            "Review the draft. TM match context and glossary hints used to generate it are listed above the draft text.",
            "Click <strong>Apply draft</strong> to copy the draft into the Target field.",
            "Edit the target text as needed, then confirm with <kbd>Ctrl+Enter</kbd>.",
          ]} />
          <Tip>
            For local models, quality improves significantly when you choose a model specifically
            trained or fine-tuned for translation. Instruction-following models (those with
            &quot;Instruct&quot; in the name) generally perform better than base models.
          </Tip>
          <Note>
            The AI draft is always a suggestion — never automatically confirmed. You remain in
            full control of what goes into the TM and the final exported file.
          </Note>
        </Section>

        {/* ── QA ── */}
        <Section title="8. Quality Assurance (QA)">
          <p>
            QA checks run automatically as you type in both Workbench and Project. Issues are
            shown in an inline panel below the target editor. Fix the issue and the warning
            clears immediately.
          </p>
          <H2>Available checks</H2>
          <table className="w-full text-xs border-collapse">
            <thead>
              <tr className="text-left bg-slate-100">
                <th className="py-1.5 px-2 border border-slate-200">Check</th>
                <th className="py-1.5 px-2 border border-slate-200">What it detects</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-100">
              <tr>
                <td className="py-1 px-2 border border-slate-200 font-medium">Missing tags</td>
                <td className="py-1 px-2 border border-slate-200">
                  XLIFF inline elements (<code>&lt;g&gt;</code>, <code>&lt;x/&gt;</code>,
                  <code>&lt;bx/&gt;</code>, etc.) present in the source but absent from the target.
                  Dropping tags typically breaks the formatting of the exported document.
                </td>
              </tr>
              <tr>
                <td className="py-1 px-2 border border-slate-200 font-medium">Missing numbers</td>
                <td className="py-1 px-2 border border-slate-200">
                  Integers, decimals, and comma-formatted numbers (e.g. 3.2.11, 1,000, 42)
                  present in the source but absent from the target. This catches dropped
                  version numbers, prices, and measurements.
                </td>
              </tr>
              <tr>
                <td className="py-1 px-2 border border-slate-200 font-medium">Forbidden terms</td>
                <td className="py-1 px-2 border border-slate-200">
                  Glossary entries marked as Forbidden that appear in the target text. The
                  offending term is reported so you can replace it with the approved equivalent.
                </td>
              </tr>
            </tbody>
          </table>
          <Tip>
            QA issues are warnings, not blockers — you can still confirm a segment that has
            outstanding issues. Use this when a number difference is intentional (e.g. the
            target language uses a different numeral system).
          </Tip>
        </Section>

        {/* ── SETTINGS ── */}
        <Section title="9. Settings">
          <H2>Language pair</H2>
          <p>
            Enter BCP-47 language codes for the source and target languages. These codes are
            stored in TM records and included in TMX exports. Examples:
          </p>
          <table className="w-full text-xs border-collapse">
            <thead>
              <tr className="text-left bg-slate-100">
                <th className="py-1.5 px-2 border border-slate-200">Code</th>
                <th className="py-1.5 px-2 border border-slate-200">Language</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-100">
              {[["en","English"],["fr","French"],["de","German"],["ja","Japanese"],
                ["zh-hans","Chinese (Simplified)"],["zh-hant","Chinese (Traditional)"],
                ["pt-br","Portuguese (Brazil)"],["es","Spanish"]].map(([code, lang]) => (
                <tr key={code}>
                  <td className="py-1 px-2 border border-slate-200 font-mono">{code}</td>
                  <td className="py-1 px-2 border border-slate-200">{lang}</td>
                </tr>
              ))}
            </tbody>
          </table>
          <H2>TM settings</H2>
          <table className="w-full text-xs border-collapse">
            <thead>
              <tr className="text-left bg-slate-100">
                <th className="py-1.5 px-2 border border-slate-200">Setting</th>
                <th className="py-1.5 px-2 border border-slate-200">Description</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-100">
              <tr><td className="py-1 px-2 border border-slate-200">Fuzzy threshold</td><td className="py-1 px-2 border border-slate-200">Minimum similarity % for a TM match to be shown (50–100). Default: 70 %.</td></tr>
              <tr><td className="py-1 px-2 border border-slate-200">Max matches</td><td className="py-1 px-2 border border-slate-200">Maximum TM matches displayed per segment. Default: 5.</td></tr>
              <tr><td className="py-1 px-2 border border-slate-200">TM database path</td><td className="py-1 px-2 border border-slate-200">Leave blank to use the default location in your app data folder. Specify a custom path to use a shared TM on a network drive.</td></tr>
            </tbody>
          </table>
          <Note>
            Changing the TM database path does not migrate existing data. Export your TM as TMX
            first, change the path, then re-import the TMX into the new database.
          </Note>
          <H2>Migrate from Felix 2.x</H2>
          <p>
            If you previously used Felix 2.x on Windows, click <strong>Check for Felix 2.x
            settings</strong>. Felix reads the registry key written by the old installer and
            pre-fills the language pair and other compatible settings.
          </p>
        </Section>

        {/* ── KEYBOARD SHORTCUTS ── */}
        <Section title="10. Keyboard Shortcuts">
          <H2>Global</H2>
          <table className="w-full text-xs border-collapse">
            <thead>
              <tr className="text-left bg-slate-100">
                <th className="py-1.5 px-2 border border-slate-200 w-40">Shortcut</th>
                <th className="py-1.5 px-2 border border-slate-200">Action</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-100">
              <tr><td className="py-1 px-2 border border-slate-200"><Kbd>Ctrl+Enter</Kbd></td><td className="py-1 px-2 border border-slate-200">Confirm current segment and advance to the next untranslated one</td></tr>
            </tbody>
          </table>
          <H2>Project view</H2>
          <table className="w-full text-xs border-collapse">
            <thead>
              <tr className="text-left bg-slate-100">
                <th className="py-1.5 px-2 border border-slate-200 w-40">Shortcut</th>
                <th className="py-1.5 px-2 border border-slate-200">Action</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-100">
              <tr><td className="py-1 px-2 border border-slate-200"><Kbd>↑</Kbd> / <Kbd>↓</Kbd></td><td className="py-1 px-2 border border-slate-200">Move to the previous / next segment</td></tr>
              <tr><td className="py-1 px-2 border border-slate-200"><Kbd>Ctrl+Enter</Kbd></td><td className="py-1 px-2 border border-slate-200">Confirm segment, save to TM, advance to next untranslated</td></tr>
            </tbody>
          </table>
        </Section>

        {/* ── FILE FORMATS ── */}
        <Section title="11. Supported File Formats">
          <table className="w-full text-xs border-collapse">
            <thead>
              <tr className="text-left bg-slate-100">
                <th className="py-1.5 px-2 border border-slate-200">Format</th>
                <th className="py-1.5 px-2 border border-slate-200">Import (Project)</th>
                <th className="py-1.5 px-2 border border-slate-200">Export</th>
                <th className="py-1.5 px-2 border border-slate-200">Notes</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-100">
              <tr>
                <td className="py-1 px-2 border border-slate-200 font-medium">XLIFF 1.2 / 2.0</td>
                <td className="py-1 px-2 border border-slate-200">✓</td>
                <td className="py-1 px-2 border border-slate-200">✓</td>
                <td className="py-1 px-2 border border-slate-200">Industry-standard bilingual format. Exported as XLIFF 1.2.</td>
              </tr>
              <tr>
                <td className="py-1 px-2 border border-slate-200 font-medium">DOCX</td>
                <td className="py-1 px-2 border border-slate-200">✓</td>
                <td className="py-1 px-2 border border-slate-200">✓ (translated)</td>
                <td className="py-1 px-2 border border-slate-200">Exported DOCX replaces source text with translations; formatting preserved.</td>
              </tr>
              <tr>
                <td className="py-1 px-2 border border-slate-200 font-medium">XLSX</td>
                <td className="py-1 px-2 border border-slate-200">✓</td>
                <td className="py-1 px-2 border border-slate-200">—</td>
                <td className="py-1 px-2 border border-slate-200">Each non-empty cell is imported as a segment.</td>
              </tr>
              <tr>
                <td className="py-1 px-2 border border-slate-200 font-medium">HTML</td>
                <td className="py-1 px-2 border border-slate-200">✓</td>
                <td className="py-1 px-2 border border-slate-200">—</td>
                <td className="py-1 px-2 border border-slate-200">Text nodes extracted from visible elements.</td>
              </tr>
              <tr>
                <td className="py-1 px-2 border border-slate-200 font-medium">Plain text</td>
                <td className="py-1 px-2 border border-slate-200">✓</td>
                <td className="py-1 px-2 border border-slate-200">—</td>
                <td className="py-1 px-2 border border-slate-200">Split into segments by sentence boundary detection.</td>
              </tr>
              <tr>
                <td className="py-1 px-2 border border-slate-200 font-medium">PO / POT</td>
                <td className="py-1 px-2 border border-slate-200">✓</td>
                <td className="py-1 px-2 border border-slate-200">✓</td>
                <td className="py-1 px-2 border border-slate-200">GNU gettext format for software localisation.</td>
              </tr>
              <tr>
                <td className="py-1 px-2 border border-slate-200 font-medium">JSON</td>
                <td className="py-1 px-2 border border-slate-200">✓</td>
                <td className="py-1 px-2 border border-slate-200">✓</td>
                <td className="py-1 px-2 border border-slate-200">Flat key–value pairs. Nested objects are flattened with dot notation.</td>
              </tr>
              <tr>
                <td className="py-1 px-2 border border-slate-200 font-medium">CSV</td>
                <td className="py-1 px-2 border border-slate-200">✓</td>
                <td className="py-1 px-2 border border-slate-200">✓</td>
                <td className="py-1 px-2 border border-slate-200">Auto-detects column headers (id/key, source/src, target/tgt/translation, note/comment). Exported with four columns: id, source, target, note.</td>
              </tr>
              <tr>
                <td className="py-1 px-2 border border-slate-200 font-medium">TMX</td>
                <td className="py-1 px-2 border border-slate-200">✓ (TM import)</td>
                <td className="py-1 px-2 border border-slate-200">✓ (TM export)</td>
                <td className="py-1 px-2 border border-slate-200">Translation Memory eXchange format. Import/export via Memory Browser, not Project.</td>
              </tr>
            </tbody>
          </table>
          <H2>CSV column detection</H2>
          <p>
            When importing a CSV file, Felix automatically identifies columns by their header
            names (case-insensitive):
          </p>
          <Bullets items={[
            <><strong>ID / key</strong> — segment identifier</>,
            <><strong>source / src</strong> — source language text</>,
            <><strong>target / tgt / translation</strong> — existing translation (may be empty)</>,
            <><strong>note / comment</strong> — translator note (optional)</>,
          ]} />
          <Tip>
            To prepare a bilingual Excel file for import, export it as CSV with the correct
            column headers, then open the CSV in Felix. All non-empty target cells will be
            pre-populated in the Project view.
          </Tip>
        </Section>

        {/* ── WORKFLOW TIPS ── */}
        <Section title="12. Typical Workflows">
          <H2>Translating a new DOCX document from scratch</H2>
          <Steps items={[
            "Go to <strong>Project</strong> and open the DOCX file.",
            "Work through each segment. For segments with a high TM match, click the match to load it; for others, type your translation or apply the AI draft.",
            "Press <kbd>Ctrl+Enter</kbd> to confirm each segment and advance.",
            "When all segments are translated, click <strong>Export → DOCX</strong> to download the translated document.",
          ]} />
          <H2>Building up a TM over time</H2>
          <Steps items={[
            "Start with an empty TM. Every segment you confirm in Project or Workbench is automatically added.",
            "When starting a new project, import any client-provided TMX or XLIFF reference memories via <strong>Memory Browser → Import TM</strong>.",
            "As the TM grows, you will see increasing numbers of high-score matches for recurring sentences — especially headers, footers, and standard clauses.",
            "Periodically export a TMX backup from <strong>Memory Browser → Export TMX</strong>.",
          ]} />
          <H2>Using Felix alongside a Word document (Workbench mode)</H2>
          <Steps items={[
            "Open your Word document alongside Felix.",
            "Select a sentence in Word, then click <strong>Get from Word</strong> in the Workbench toolbar to load it into the Source field.",
            "Translate the segment using TM matches, glossary hits, and the AI draft.",
            "Click <strong>Confirm segment</strong> to save to the TM, then <strong>Insert into Word</strong> to paste the translation into your document.",
            "Repeat for each sentence.",
          ]} />
          <H2>Terminology management with a client glossary</H2>
          <Steps items={[
            "Receive a glossary from the client in CSV or Excel format.",
            "If it is an Excel file, export it as CSV with columns: source, target, note.",
            "Go to the <strong>Glossary</strong> view and import the CSV, or add entries manually.",
            "Mark any terms the client has explicitly banned as <strong>Forbidden</strong>.",
            "As you translate, glossary hits appear automatically. QA will flag any forbidden term that slips into a target segment.",
          ]} />
        </Section>

      </div>
    </div>
  );
}
