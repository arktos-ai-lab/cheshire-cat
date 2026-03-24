import { useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import {
  glossaryCount,
  glossaryDelete,
  glossaryInsert,
  glossaryListAll,
} from "../api";
import type { GlossaryTerm } from "../types";

interface NewTermForm {
  sourceTerm: string;
  targetTerm: string;
  domain: string;
  note: string;
  forbidden: boolean;
}

const EMPTY_FORM: NewTermForm = {
  sourceTerm: "",
  targetTerm: "",
  domain: "",
  note: "",
  forbidden: false,
};

export default function GlossaryManager() {
  const qc = useQueryClient();
  const [form, setForm] = useState<NewTermForm>(EMPTY_FORM);
  const [error, setError] = useState("");

  const { data: count = 0 } = useQuery({
    queryKey: ["glossary_count"],
    queryFn: glossaryCount,
  });

  const { data: terms = [] } = useQuery<GlossaryTerm[]>({
    queryKey: ["glossary_all"],
    queryFn: glossaryListAll,
  });

  const insertMut = useMutation({
    mutationFn: () =>
      glossaryInsert({
        sourceTerm: form.sourceTerm,
        targetTerm: form.targetTerm,
        domain: form.domain || undefined,
        note: form.note || undefined,
        forbidden: form.forbidden,
      }),
    onSuccess: () => {
      setForm(EMPTY_FORM);
      setError("");
      qc.invalidateQueries({ queryKey: ["glossary_count"] });
      qc.invalidateQueries({ queryKey: ["glossary_all"] });
    },
    onError: (err: unknown) => {
      setError(String(err));
    },
  });

  const deleteMut = useMutation({
    mutationFn: glossaryDelete,
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["glossary_count"] });
      qc.invalidateQueries({ queryKey: ["glossary_all"] });
    },
  });

  function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!form.sourceTerm.trim() || !form.targetTerm.trim()) {
      setError("Source term and target term are required.");
      return;
    }
    insertMut.mutate();
  }

  return (
    <div className="flex h-full overflow-hidden">
      {/* ── Add term form ── */}
      <aside className="w-72 border-r border-slate-200 bg-white p-4 shrink-0 overflow-y-auto">
        <h2 className="text-sm font-semibold text-slate-800 mb-4">
          Add Glossary Term
        </h2>

        <form onSubmit={handleSubmit} className="space-y-3">
          <div>
            <label className="label">Source term *</label>
            <input
              className="input"
              value={form.sourceTerm}
              onChange={(e) =>
                setForm((f) => ({ ...f, sourceTerm: e.target.value }))
              }
              placeholder="e.g. submission"
            />
          </div>

          <div>
            <label className="label">Target term *</label>
            <input
              className="input"
              value={form.targetTerm}
              onChange={(e) =>
                setForm((f) => ({ ...f, targetTerm: e.target.value }))
              }
              placeholder="e.g. 提出"
            />
          </div>

          <div>
            <label className="label">Domain</label>
            <input
              className="input"
              value={form.domain}
              onChange={(e) =>
                setForm((f) => ({ ...f, domain: e.target.value }))
              }
              placeholder="e.g. Legal"
            />
          </div>

          <div>
            <label className="label">Note</label>
            <textarea
              className="input h-16"
              value={form.note}
              onChange={(e) =>
                setForm((f) => ({ ...f, note: e.target.value }))
              }
              placeholder="Optional usage note"
            />
          </div>

          <div className="space-y-1">
            <div className="flex items-center gap-2">
              <input
                type="checkbox"
                id="forbidden"
                checked={form.forbidden}
                onChange={(e) =>
                  setForm((f) => ({ ...f, forbidden: e.target.checked }))
                }
                className="rounded border-slate-300 text-red-500"
              />
              <label htmlFor="forbidden" className="text-sm text-slate-600">
                Forbidden term
              </label>
            </div>
            <p className="text-xs text-slate-400 pl-6">
              QA will flag any segment where this term appears in the target.
              Use for banned words, deprecated names, or client style violations.
            </p>
          </div>

          {error && (
            <p className="text-xs text-red-600">{error}</p>
          )}

          <button
            type="submit"
            className="btn-primary w-full justify-center"
            disabled={insertMut.isPending}
          >
            {insertMut.isPending ? "Adding…" : "Add term"}
          </button>
        </form>
      </aside>

      {/* ── Terms list ── */}
      <div className="flex flex-col flex-1 min-w-0 p-4 gap-3 overflow-hidden">
        <div className="flex items-center justify-between shrink-0">
          <h1 className="text-lg font-semibold text-slate-800">Glossary</h1>
          <span className="text-sm text-slate-500">
            {count.toLocaleString()} terms
          </span>
        </div>

        <div className="card flex-1 overflow-auto">
          {terms.length === 0 ? (
            <div className="flex items-center justify-center h-full text-sm text-slate-400 p-8 text-center">
              <div>
                <p className="mb-1">No terms to display.</p>
                <p className="text-xs">
                  Glossary hits appear in the Workbench as you type.
                  Use the form on the left to add new terms.
                </p>
              </div>
            </div>
          ) : (
            <table className="w-full text-xs">
              <thead className="bg-slate-50 border-b border-slate-200 sticky top-0">
                <tr>
                  <th className="text-left px-3 py-2 font-medium text-slate-600">
                    Source
                  </th>
                  <th className="text-left px-3 py-2 font-medium text-slate-600">
                    Target
                  </th>
                  <th className="text-left px-3 py-2 font-medium text-slate-600">
                    Domain
                  </th>
                  <th className="text-left px-3 py-2 font-medium text-slate-600">
                    Status
                  </th>
                  <th className="px-3 py-2 w-16" />
                </tr>
              </thead>
              <tbody className="divide-y divide-slate-100">
                {terms.map((t) => (
                  <tr key={t.id} className="hover:bg-slate-50 group">
                    <td className="px-3 py-2 text-slate-700">{t.source_term}</td>
                    <td className="px-3 py-2 text-slate-800 font-medium">
                      {t.target_term}
                    </td>
                    <td className="px-3 py-2 text-slate-500">
                      {t.domain ?? "—"}
                    </td>
                    <td className="px-3 py-2">
                      {t.forbidden ? (
                        <span className="text-xs text-red-600 font-medium">
                          Forbidden
                        </span>
                      ) : (
                        <span className="text-xs text-emerald-600">Active</span>
                      )}
                    </td>
                    <td className="px-3 py-2 text-right">
                      <button
                        className="btn-danger opacity-0 group-hover:opacity-100 transition-opacity py-0.5 px-2 text-xs"
                        onClick={() => deleteMut.mutate(t.id)}
                      >
                        Delete
                      </button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </div>
      </div>
    </div>
  );
}
