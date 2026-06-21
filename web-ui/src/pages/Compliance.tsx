import { useEffect, useState } from "react";
import { fetchCompliance, runCompliance } from "../api";

export function CompliancePage() {
  const [reports, setReports] = useState<Array<Record<string, unknown>>>([]);
  const [error, setError] = useState<string | null>(null);
  const [running, setRunning] = useState(false);

  function reload() {
    fetchCompliance().then(setReports).catch((e: Error) => setError(e.message));
  }

  useEffect(() => {
    reload();
  }, []);

  async function onRun() {
    setRunning(true);
    try {
      const result = await runCompliance();
      setReports(result);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed");
    } finally {
      setRunning(false);
    }
  }

  if (error) return <p className="text-red-400">{error}</p>;

  return (
    <div>
      <div className="flex items-center justify-between mb-6">
        <h1 className="text-2xl font-semibold">Compliance</h1>
        <button
          onClick={onRun}
          disabled={running}
          className="rounded bg-violet-700 px-4 py-2 text-sm disabled:opacity-50"
        >
          {running ? "Running…" : "Run checks"}
        </button>
      </div>
      <div className="space-y-2">
        {reports.map((r) => (
          <div key={String(r.id)} className="rounded border border-slate-800 p-3">
            <div className="flex justify-between">
              <span className="font-medium">{String(r.check_type)}</span>
              <span
                className={
                  r.status === "passed"
                    ? "text-green-400"
                    : r.status === "failed"
                      ? "text-red-400"
                      : "text-yellow-400"
                }
              >
                {String(r.status)}
              </span>
            </div>
            <p className="text-sm text-slate-400 mt-1">{String(r.summary)}</p>
          </div>
        ))}
      </div>
    </div>
  );
}
