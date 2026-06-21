import { useEffect, useState } from "react";
import { fetchCloudXdrAnalytics, type XdrAnalyticsSummary } from "../api";

export function XdrFleetAnalyticsPage() {
  const [summary, setSummary] = useState<XdrAnalyticsSummary | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    fetchCloudXdrAnalytics()
      .then(setSummary)
      .catch((e: Error) => setError(e.message));
  }, []);

  if (error) return <p className="text-red-400">{error}</p>;
  if (!summary) return <p>Loading XDR fleet analytics…</p>;

  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-semibold">XDR Fleet Analytics</h1>
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        <Stat label="Total incidents" value={summary.total_incidents} />
        <Stat label="Open incidents" value={summary.open_incidents} />
        <Stat label="Critical incidents" value={summary.critical_incidents} />
        <Stat label="Total detections" value={summary.total_detections} />
        <Stat label="MITRE techniques" value={summary.mitre_techniques_detected} />
        <Stat label="MITRE coverage" value={Math.round(summary.mitre_coverage_pct)} suffix="%" />
        <Stat label="Avg MTTR (hours)" value={Math.round(summary.avg_incident_mttr_hours * 10) / 10} />
        <Stat label="Fleet threat score" value={Math.round(summary.fleet_threat_score)} />
        <Stat label="Rollups recorded" value={summary.rollups_recorded} />
      </div>
    </div>
  );
}

function Stat({ label, value, suffix = "" }: { label: string; value: number; suffix?: string }) {
  return (
    <div className="rounded-lg border border-slate-800 bg-slate-900 p-4">
      <div className="text-sm text-slate-400">{label}</div>
      <div className="text-3xl font-bold mt-2">{value}{suffix}</div>
    </div>
  );
}
