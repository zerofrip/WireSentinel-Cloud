import { useEffect, useState } from "react";
import { fetchCloudAnonymity, type AnonymityFleetOverview } from "../api";

export function FederationMonitoringPage() {
  const [overview, setOverview] = useState<AnonymityFleetOverview | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    fetchCloudAnonymity()
      .then(setOverview)
      .catch((e: Error) => setError(e.message));
  }, []);

  if (error) return <p className="text-red-400">{error}</p>;
  if (!overview) return <p>Loading federation monitoring…</p>;

  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-semibold">Federation Monitoring</h1>
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        <Stat label="Controllers reporting" value={overview.controllers_reporting} />
        <Stat label="Connected devices" value={overview.connected_devices} />
        <Stat label="Federation peers" value={overview.federation_peers_total} />
      </div>
      <div className="rounded-lg border border-slate-800 bg-slate-900 p-4">
        <h2 className="text-sm font-medium text-slate-400 mb-3">Fleet health</h2>
        <p className="text-sm text-slate-300">
          {overview.healthy_devices} of {overview.reporting_devices} reporting devices healthy across{" "}
          {overview.controllers_reporting} controller(s).
        </p>
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
