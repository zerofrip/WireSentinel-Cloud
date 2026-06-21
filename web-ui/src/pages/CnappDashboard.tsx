import { useEffect, useState } from "react";
import { fetchCloudCnapp, type CnappFleetOverview } from "../api";

export function CnappDashboardPage() {
  const [overview, setOverview] = useState<CnappFleetOverview | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    fetchCloudCnapp()
      .then(setOverview)
      .catch((e: Error) => setError(e.message));
  }, []);

  if (error) return <p className="text-red-400">{error}</p>;
  if (!overview) return <p>Loading CNAPP dashboard…</p>;

  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-semibold">CNAPP Dashboard</h1>
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <Stat label="Fleet risk score" value={Math.round(overview.fleet_risk_score)} />
        <Stat label="Posture score" value={Math.round(overview.posture_score)} />
        <Stat label="Compliance" value={Math.round(overview.compliance_pct)} suffix="%" />
        <Stat label="Reporting accounts" value={overview.reporting_accounts} />
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
