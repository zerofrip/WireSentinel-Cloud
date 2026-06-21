import { useEffect, useState } from "react";
import { fetchCloudSse, type SseFleetOverview } from "../api";

export function ThreatCenterPage() {
  const [overview, setOverview] = useState<SseFleetOverview | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    fetchCloudSse()
      .then(setOverview)
      .catch((e: Error) => setError(e.message));
  }, []);

  if (error) return <p className="text-red-400">{error}</p>;
  if (!overview) return <p>Loading threat center…</p>;

  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-semibold">Threat Center</h1>
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <Stat label="Threat matches" value={overview.threat_count} />
        <Stat label="SWG blocked" value={overview.swg_blocked} />
        <Stat label="Reporting devices" value={overview.reporting_devices} />
        <Stat label="Controllers" value={overview.controllers_reporting} />
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
