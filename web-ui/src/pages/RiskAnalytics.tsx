import { useEffect, useState } from "react";
import { fetchCloudSseAnalytics, type SseAnalyticsSummary } from "../api";

export function RiskAnalyticsPage() {
  const [summary, setSummary] = useState<SseAnalyticsSummary | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    fetchCloudSseAnalytics()
      .then(setSummary)
      .catch((e: Error) => setError(e.message));
  }, []);

  if (error) return <p className="text-red-400">{error}</p>;
  if (!summary) return <p>Loading risk analytics…</p>;

  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-semibold">Risk Analytics</h1>
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        <Stat label="Avg risk score" value={Math.round(summary.avg_risk_score)} />
        <Stat label="Threat count" value={summary.threat_count} />
        <Stat label="Block ratio" value={Math.round(summary.block_ratio * 100)} suffix="%" />
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
