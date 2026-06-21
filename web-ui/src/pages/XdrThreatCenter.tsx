import { useEffect, useState } from "react";
import { fetchCloudXdr, type XdrFleetOverview } from "../api";

export function XdrThreatCenterPage() {
  const [overview, setOverview] = useState<XdrFleetOverview | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    fetchCloudXdr()
      .then(setOverview)
      .catch((e: Error) => setError(e.message));
  }, []);

  if (error) return <p className="text-red-400">{error}</p>;
  if (!overview) return <p>Loading XDR threat center…</p>;

  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-semibold">XDR Threat Center</h1>
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <Stat label="Fleet threat score" value={Math.round(overview.fleet_threat_score)} />
        <Stat label="Open incidents" value={overview.open_incidents} />
        <Stat label="Critical incidents" value={overview.critical_incidents} />
        <Stat label="Reporting devices" value={overview.reporting_devices} />
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
