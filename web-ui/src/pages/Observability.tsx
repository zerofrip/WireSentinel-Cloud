import { useEffect, useState } from "react";
import { fetchLogs, fetchObservabilityMetrics } from "../api";

export function ObservabilityPage() {
  const [metrics, setMetrics] = useState<Record<string, unknown> | null>(null);
  const [logs, setLogs] = useState<Array<Record<string, unknown>>>([]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    Promise.all([fetchObservabilityMetrics(), fetchLogs(50)])
      .then(([m, l]) => {
        setMetrics(m);
        setLogs(l);
      })
      .catch((e: Error) => setError(e.message));
  }, []);

  if (error) return <p className="text-red-400">{error}</p>;
  if (!metrics) return <p>Loading…</p>;

  const records = (metrics.records as Array<Record<string, unknown>>) ?? [];
  const exporters = (metrics.exporters as Array<Record<string, unknown>>) ?? [];

  return (
    <div>
      <h1 className="text-2xl font-semibold mb-6">Observability</h1>
      <h2 className="text-lg mb-3">Exporters</h2>
      <div className="flex gap-3 mb-6">
        {exporters.map((e) => (
          <span
            key={String(e.name)}
            className={`text-xs px-2 py-1 rounded ${e.enabled ? "bg-green-900 text-green-100" : "bg-slate-800"}`}
          >
            {String(e.name)}{e.stub ? " (stub)" : ""}
          </span>
        ))}
      </div>
      <h2 className="text-lg mb-3">Telemetry records</h2>
      <div className="grid md:grid-cols-2 gap-3 mb-8">
        {records.map((r) => (
          <div key={String(r.name)} className="rounded border border-slate-800 p-3 text-sm">
            <div className="text-slate-400">{String(r.name)}</div>
            <div className="text-xl font-bold">{String(r.value)}</div>
          </div>
        ))}
      </div>
      <h2 className="text-lg mb-3">Recent logs</h2>
      <ul className="space-y-1 text-sm font-mono">
        {logs.map((l) => (
          <li key={String(l.id)} className="text-slate-300">
            [{String(l.level)}] {String(l.source)}: {String(l.message)}
          </li>
        ))}
      </ul>
    </div>
  );
}
