import { useEffect, useState } from "react";
import { fetchCloudAnonymity, type AnonymityFleetOverview } from "../api";

export function AnonymityFleetPage() {
  const [overview, setOverview] = useState<AnonymityFleetOverview | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    fetchCloudAnonymity()
      .then(setOverview)
      .catch((e: Error) => setError(e.message));
  }, []);

  if (error) return <p className="text-red-400">{error}</p>;
  if (!overview) return <p>Loading anonymity fleet…</p>;

  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-semibold">Anonymity Fleet</h1>
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <Stat label="Reporting" value={overview.reporting_devices} />
        <Stat label="Healthy" value={overview.healthy_devices} />
        <Stat label="Federation peers" value={overview.federation_peers_total} />
        <Stat label="Avg score" value={Math.round(overview.avg_anonymity_score)} />
      </div>
      <div className="rounded-lg border border-slate-800 bg-slate-900 p-4">
        <h2 className="text-sm font-medium text-slate-400 mb-3">Controller rollups</h2>
        {overview.rollups.length === 0 ? (
          <p className="text-sm text-slate-500">No anonymity rollups recorded yet</p>
        ) : (
          <ul className="space-y-2 text-sm">
            {overview.rollups.map((r) => (
              <li key={r.id} className="flex justify-between gap-4 p-2 rounded bg-slate-800/50">
                <span>{r.controller_id ?? "fleet"}</span>
                <span className="text-slate-400">
                  score {r.avg_anonymity_score.toFixed(0)} · {r.healthy_devices}/{r.reporting_devices}{" "}
                  healthy · {r.rolled_up_at}
                </span>
              </li>
            ))}
          </ul>
        )}
      </div>
    </div>
  );
}

function Stat({ label, value }: { label: string; value: number }) {
  return (
    <div className="rounded-lg border border-slate-800 bg-slate-900 p-4">
      <div className="text-sm text-slate-400">{label}</div>
      <div className="text-3xl font-bold mt-2">{value}</div>
    </div>
  );
}
