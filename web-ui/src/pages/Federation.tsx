import { useEffect, useState } from "react";
import { fetchSync } from "../api";

export function FederationPage() {
  const [entities, setEntities] = useState<unknown[]>([]);
  const [conflicts, setConflicts] = useState<unknown[]>([]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    fetchSync()
      .then((data) => {
        setEntities(data.entities);
        setConflicts(data.conflicts);
      })
      .catch((e: Error) => setError(e.message));
  }, []);

  if (error) return <p className="text-red-400">{error}</p>;

  return (
    <div>
      <h1 className="text-2xl font-semibold mb-6">Federation Sync</h1>
      <div className="grid md:grid-cols-2 gap-4">
        <div className="rounded border border-slate-800 p-4">
          <h2 className="font-medium mb-2">Synced entities ({entities.length})</h2>
          <pre className="text-xs text-slate-400 overflow-auto max-h-64">
            {JSON.stringify(entities, null, 2)}
          </pre>
        </div>
        <div className="rounded border border-slate-800 p-4">
          <h2 className="font-medium mb-2">Open conflicts ({conflicts.length})</h2>
          <pre className="text-xs text-slate-400 overflow-auto max-h-64">
            {JSON.stringify(conflicts, null, 2)}
          </pre>
        </div>
      </div>
    </div>
  );
}
