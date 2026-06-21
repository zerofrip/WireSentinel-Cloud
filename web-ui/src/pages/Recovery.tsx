import { useEffect, useState } from "react";
import { fetchRecoveryRuns } from "../api";

export function RecoveryPage() {
  const [runs, setRuns] = useState<Array<Record<string, unknown>>>([]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    fetchRecoveryRuns()
      .then(setRuns)
      .catch((e: Error) => setError(e.message));
  }, []);

  if (error) return <p className="text-red-400">{error}</p>;

  return (
    <div>
      <h1 className="text-2xl font-semibold mb-6">Disaster Recovery</h1>
      <ul className="space-y-2">
        {runs.map((run) => (
          <li key={String(run.id)} className="rounded border border-slate-800 p-3">
            Plan {String(run.plan_id)} · {String(run.status)}
            {run.started_at ? (
              <span className="text-slate-400 text-sm ml-2">started {String(run.started_at)}</span>
            ) : null}
          </li>
        ))}
        {runs.length === 0 && <li className="text-slate-400 text-sm">No recovery runs</li>}
      </ul>
    </div>
  );
}
