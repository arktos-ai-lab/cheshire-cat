import { useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import {
  tmCount,
  tmList,
  tmDeleteRecord,
  tmImport,
  tmExport,
  openTmxFile,
  saveTmxFile,
} from "../api";
import type { ImportStats } from "../types";

const PAGE_SIZE = 50;

export default function MemoryBrowser() {
  const qc = useQueryClient();
  const [page, setPage] = useState(0);
  const [importResult, setImportResult] = useState<ImportStats | null>(null);
  const [statusMsg, setStatusMsg] = useState("");

  const { data: count = 0 } = useQuery({
    queryKey: ["tm_count"],
    queryFn: tmCount,
  });

  const { data: records = [], isFetching } = useQuery({
    queryKey: ["tm_list", page],
    queryFn: () => tmList(PAGE_SIZE, page * PAGE_SIZE),
  });

  const deleteMut = useMutation({
    mutationFn: tmDeleteRecord,
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["tm_list"] });
      qc.invalidateQueries({ queryKey: ["tm_count"] });
    },
  });

  async function handleImport() {
    const path = await openTmxFile();
    if (!path) return;
    try {
      const stats = await tmImport(path);
      setImportResult(stats);
      qc.invalidateQueries({ queryKey: ["tm_list"] });
      qc.invalidateQueries({ queryKey: ["tm_count"] });
    } catch (err) {
      setStatusMsg(`Import failed: ${err}`);
    }
  }

  async function handleExport() {
    const path = await saveTmxFile();
    if (!path) return;
    try {
      await tmExport(path);
      setStatusMsg("Export complete.");
    } catch (err) {
      setStatusMsg(`Export failed: ${err}`);
    }
  }

  const totalPages = Math.ceil(count / PAGE_SIZE);

  return (
    <div className="flex flex-col h-full p-4 gap-4 overflow-hidden">
      {/* Header */}
      <div className="flex items-center justify-between shrink-0">
        <div>
          <h1 className="text-lg font-semibold text-slate-800">
            Translation Memory
          </h1>
          <p className="text-sm text-slate-500">{count.toLocaleString()} records</p>
        </div>
        <div className="flex gap-2">
          <button className="btn-secondary" onClick={handleImport}>
            Import TMX
          </button>
          <button className="btn-primary" onClick={handleExport}>
            Export TMX
          </button>
        </div>
      </div>

      {/* Import result */}
      {importResult && (
        <div className="shrink-0 card p-3 text-sm text-slate-700">
          Import complete — {importResult.imported} imported,{" "}
          {importResult.skipped_duplicates} duplicates skipped,{" "}
          {importResult.skipped_errors} errors.
          <button
            className="ml-3 text-xs text-slate-400 hover:text-slate-600"
            onClick={() => setImportResult(null)}
          >
            Dismiss
          </button>
        </div>
      )}

      {statusMsg && (
        <div className="shrink-0 card p-3 text-sm text-slate-700">
          {statusMsg}
          <button
            className="ml-3 text-xs text-slate-400 hover:text-slate-600"
            onClick={() => setStatusMsg("")}
          >
            Dismiss
          </button>
        </div>
      )}

      {/* Records table */}
      <div className="flex-1 card overflow-hidden flex flex-col min-h-0">
        <div className="overflow-auto flex-1">
          <table className="w-full text-xs">
            <thead className="bg-slate-50 border-b border-slate-200 sticky top-0">
              <tr>
                <th className="text-left px-3 py-2 font-medium text-slate-600 w-1/2">
                  Source
                </th>
                <th className="text-left px-3 py-2 font-medium text-slate-600 w-1/2">
                  Target
                </th>
                <th className="px-3 py-2 font-medium text-slate-600 w-16" />
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-100">
              {isFetching && records.length === 0 ? (
                <tr>
                  <td colSpan={3} className="px-3 py-4 text-center text-slate-400">
                    Loading…
                  </td>
                </tr>
              ) : records.length === 0 ? (
                <tr>
                  <td colSpan={3} className="px-3 py-8 text-center text-slate-400">
                    No records. Import a TMX file to get started.
                  </td>
                </tr>
              ) : (
                records.map((r) => (
                  <tr key={r.id} className="hover:bg-slate-50 group">
                    <td className="px-3 py-2 text-slate-700 break-words max-w-0 w-1/2">
                      {r.source}
                    </td>
                    <td className="px-3 py-2 text-slate-800 break-words max-w-0 w-1/2">
                      {r.target}
                    </td>
                    <td className="px-3 py-2 text-right">
                      <button
                        className="btn-danger opacity-0 group-hover:opacity-100 transition-opacity py-0.5 px-2 text-xs"
                        onClick={() => deleteMut.mutate(r.id)}
                      >
                        Delete
                      </button>
                    </td>
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </div>

        {/* Pagination */}
        {totalPages > 1 && (
          <div className="flex items-center justify-between px-3 py-2 border-t border-slate-200 shrink-0">
            <button
              className="btn-secondary py-1"
              disabled={page === 0}
              onClick={() => setPage((p) => p - 1)}
            >
              Previous
            </button>
            <span className="text-xs text-slate-500">
              Page {page + 1} of {totalPages}
            </span>
            <button
              className="btn-secondary py-1"
              disabled={page >= totalPages - 1}
              onClick={() => setPage((p) => p + 1)}
            >
              Next
            </button>
          </div>
        )}
      </div>
    </div>
  );
}
