import { useEffect, useState } from "react";
import { fetchCloudSseAnalytics, type SseAnalyticsSummary } from "../api";

export function UebaAnalyticsPage() {
  const [summary, setSummary] = useState<SseAnalyticsSummary | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    fetchCloudSseAnalytics()
      .then(setSummary)
      .catch((e: Error) => setError(e.message));
  }, []);

  if (error) return <p className="text-red-400">{error}</p>;
  if (!summary) return <p>Loading UEBA analytics…</p>;

  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-semibold">UEBA Analytics</h1>
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        <Stat label="UEBA alerts" value={summary.ueba_alerts} />
        <Stat label="Threat count" value={summary.threat_count} />
        <Stat label="Rollups recorded" value={summary.rollups_recorded} />
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
