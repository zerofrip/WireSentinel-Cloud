import { useEffect, useState } from "react";
import { fetchCloudZtnaAnalytics, type ZtnaAnalyticsSummary } from "../api";

export function TrustAnalyticsPage() {
  const [analytics, setAnalytics] = useState<ZtnaAnalyticsSummary | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    fetchCloudZtnaAnalytics()
      .then(setAnalytics)
      .catch((e: Error) => setError(e.message));
  }, []);

  if (error) return <p className="text-red-400">{error}</p>;
  if (!analytics) return <p>Loading trust analytics…</p>;

  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-semibold">Trust Analytics</h1>
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <Stat label="Allow" value={analytics.allow_count} />
        <Stat label="Deny" value={analytics.deny_count} />
        <Stat label="Challenge" value={analytics.challenge_count} />
        <Stat label="Deny ratio" value={Math.round(analytics.deny_ratio * 100)} suffix="%" />
      </div>
      <p className="text-sm text-slate-400">
        Average trust score: {analytics.avg_trust_score.toFixed(1)} · Rollups recorded:{" "}
        {analytics.rollups_recorded}
      </p>
    </div>
  );
}

function Stat({ label, value, suffix = "" }: { label: string; value: number; suffix?: string }) {
  return (
    <div className="rounded-lg border border-slate-800 bg-slate-900 p-4">
      <div className="text-sm text-slate-400">{label}</div>
      <div className="text-3xl font-bold mt-2">
        {value}
        {suffix}
      </div>
    </div>
  );
}
