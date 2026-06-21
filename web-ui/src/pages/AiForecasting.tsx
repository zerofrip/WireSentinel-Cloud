import { useEffect, useState } from "react";
import { fetchCloudAiAnalytics, type AiAnalyticsSummary } from "../api";

export function AiForecastingPage() {
  const [summary, setSummary] = useState<AiAnalyticsSummary | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    fetchCloudAiAnalytics()
      .then(setSummary)
      .catch((e: Error) => setError(e.message));
  }, []);

  if (error) return <p className="text-red-400">{error}</p>;
  if (!summary) return <p>Loading AI forecasting analytics…</p>;

  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-semibold">AI Risk Forecasting</h1>
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <Stat label="Prompt injection events" value={summary.prompt_injection_events} />
        <Stat label="Data exfiltration events" value={summary.data_exfiltration_events} />
        <Stat label="Total correlations" value={summary.total_correlations} />
        <Stat label="Avg risk score" value={Math.round(summary.avg_risk_score)} />
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
