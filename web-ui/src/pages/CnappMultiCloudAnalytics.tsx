import { useEffect, useState } from "react";
import { fetchCloudCnappAnalytics, type CnappAnalyticsSummary } from "../api";

export function CnappMultiCloudAnalyticsPage() {
  const [summary, setSummary] = useState<CnappAnalyticsSummary | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    fetchCloudCnappAnalytics()
      .then(setSummary)
      .catch((e: Error) => setError(e.message));
  }, []);

  if (error) return <p className="text-red-400">{error}</p>;
  if (!summary) return <p>Loading CNAPP multi-cloud analytics…</p>;

  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-semibold">CNAPP Multi-Cloud Analytics</h1>
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <Stat label="Open vulnerabilities" value={summary.open_vulnerabilities} />
        <Stat label="Critical vulnerabilities" value={summary.critical_vulnerabilities} />
        <Stat label="Attack paths" value={summary.attack_paths_detected} />
        <Stat label="Cloud providers" value={summary.multi_cloud_providers} />
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
