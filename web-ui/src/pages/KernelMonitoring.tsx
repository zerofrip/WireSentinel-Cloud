import { useEffect, useState } from "react";
import { fetchCloudKernel, fetchCloudKernelStatistics, type KernelFleetOverview, type KernelFleetStatistics } from "../api";

export function KernelMonitoringPage() {
  const [overview, setOverview] = useState<KernelFleetOverview | null>(null);
  const [stats, setStats] = useState<KernelFleetStatistics | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    Promise.all([fetchCloudKernel(), fetchCloudKernelStatistics()])
      .then(([o, s]) => {
        setOverview(o);
        setStats(s);
      })
      .catch((e: Error) => setError(e.message));
  }, []);

  if (error) return <p className="text-red-400">{error}</p>;
  if (!overview || !stats) return <p>Loading kernel monitoring…</p>;

  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-semibold">Kernel Monitoring</h1>
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <Stat label="Reporting devices" value={overview.reporting_devices} />
        <Stat label="Healthy" value={overview.healthy_devices} />
        <Stat label="Classify total" value={stats.classify_count} />
        <Stat label="Packets/s" value={stats.packets_per_sec} />
      </div>
      <div className="rounded-lg border border-slate-800 bg-slate-900 p-4 text-sm text-slate-400">
        Healthy ratio {(stats.avg_healthy_ratio * 100).toFixed(1)}% · kernel adoption{" "}
        {(stats.kernel_adoption_ratio * 100).toFixed(1)}% · {stats.rollups_recorded} rollups
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
