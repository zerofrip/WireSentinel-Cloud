import { useEffect, useState } from "react";
import { fetchCloudAnonymityAnalytics, type AnonymityPrivacyAnalytics } from "../api";

export function AnonymityPrivacyAnalyticsPage() {
  const [analytics, setAnalytics] = useState<AnonymityPrivacyAnalytics | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    fetchCloudAnonymityAnalytics()
      .then(setAnalytics)
      .catch((e: Error) => setError(e.message));
  }, []);

  if (error) return <p className="text-red-400">{error}</p>;
  if (!analytics) return <p>Loading privacy analytics…</p>;

  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-semibold">Anonymity Privacy Analytics</h1>
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <Stat label="Avg entropy bits" value={analytics.avg_entropy_bits.toFixed(1)} />
        <Stat label="Route entropy" value={analytics.avg_route_entropy.toFixed(2)} />
        <Stat label="Anonymity score" value={analytics.avg_anonymity_score.toFixed(0)} />
        <Stat label="Healthy ratio" value={`${(analytics.healthy_ratio * 100).toFixed(0)}%`} />
      </div>
      <div className="rounded-lg border border-slate-800 bg-slate-900 p-4 text-sm text-slate-400">
        {analytics.rollups_recorded} rollup(s) · {analytics.federation_peers_total} federation peers
        tracked
      </div>
    </div>
  );
}

function Stat({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-lg border border-slate-800 bg-slate-900 p-4">
      <div className="text-sm text-slate-400">{label}</div>
      <div className="text-3xl font-bold mt-2">{value}</div>
    </div>
  );
}
