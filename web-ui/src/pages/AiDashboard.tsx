import { useEffect, useState } from "react";
import { fetchCloudAi, type AiFleetOverview } from "../api";

export function AiDashboardPage() {
  const [overview, setOverview] = useState<AiFleetOverview | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    fetchCloudAi()
      .then(setOverview)
      .catch((e: Error) => setError(e.message));
  }, []);

  if (error) return <p className="text-red-400">{error}</p>;
  if (!overview) return <p>Loading AI security dashboard…</p>;

  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-semibold">AI Security Dashboard</h1>
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <Stat label="Fleet AI risk score" value={Math.round(overview.fleet_ai_risk_score)} />
        <Stat label="Open investigations" value={overview.open_investigations} />
        <Stat label="Compliance" value={Math.round(overview.compliance_pct)} suffix="%" />
        <Stat label="Reporting agents" value={overview.reporting_agents} />
      </div>
    </div>
  );
}

function Stat({ label, value, suffix }: { label: string; value: number; suffix?: string }) {
  return (
    <div className="rounded-lg border border-slate-800 bg-slate-900 p-4">
      <div className="text-sm text-slate-400">{label}</div>
      <div className="text-3xl font-bold mt-2">
        {value}
        {suffix ?? ""}
      </div>
    </div>
  );
}
