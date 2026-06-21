import { useEffect, useState } from "react";
import { fetchRegionHealth, fetchRegions } from "../api";

export function RegionsPage() {
  const [regions, setRegions] = useState<Array<Record<string, unknown>>>([]);
  const [health, setHealth] = useState<Array<Record<string, unknown>>>([]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    Promise.all([fetchRegions(), fetchRegionHealth()])
      .then(([r, h]) => {
        setRegions(r);
        setHealth(h);
      })
      .catch((e: Error) => setError(e.message));
  }, []);

  if (error) return <p className="text-red-400">{error}</p>;

  return (
    <div>
      <h1 className="text-2xl font-semibold mb-6">Regions</h1>
      <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-4">
        {regions.map((r) => (
          <div key={String(r.id)} className="rounded border border-slate-800 p-4">
            <div className="font-semibold">{String(r.display_name)}</div>
            <div className="text-sm text-slate-400">{String(r.name)} · {String(r.status)}</div>
            <div className="mt-2 text-sm">
              {r.healthy ? "✓ Healthy" : "⚠ Degraded"}
              {r.latency_ms != null && ` · ${r.latency_ms}ms`}
            </div>
          </div>
        ))}
      </div>
      <h2 className="text-lg mt-8 mb-3">Latest probes</h2>
      <ul className="space-y-2 text-sm">
        {health.map((h) => (
          <li key={String(h.region_id)} className="rounded border border-slate-800 p-2">
            {String(h.region_id)} — {h.healthy ? "healthy" : "unhealthy"} @ {String(h.checked_at)}
          </li>
        ))}
      </ul>
    </div>
  );
}
