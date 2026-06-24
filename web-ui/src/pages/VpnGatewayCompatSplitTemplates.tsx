import { useEffect, useState } from "react";
import { fetchCloudSplitTemplates, type WiresockFleetOverview } from "../api";

export function WiresockSplitTemplatesPage() {
  const [overview, setOverview] = useState<WiresockFleetOverview | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    fetchCloudSplitTemplates()
      .then(setOverview)
      .catch((e: Error) => setError(e.message));
  }, []);

  if (error) return <p className="text-red-400">{error}</p>;
  if (!overview) return <p>Loading split tunnel templates…</p>;

  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-semibold">Split Tunnel Templates</h1>
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <Stat label="Fleet health" value={Math.round(overview.fleet_health_score)} />
        <Stat label="Active templates" value={overview.active_split_templates} />
        <Stat label="Reporting endpoints" value={overview.reporting_endpoints} />
        <Stat label="Controllers" value={overview.controllers_reporting} />
      </div>
      <div className="overflow-x-auto">
        <table className="w-full text-sm">
          <thead>
            <tr className="text-left text-slate-400 border-b border-slate-800">
              <th className="py-2 pr-4">Name</th>
              <th className="py-2 pr-4">Mode</th>
              <th className="py-2 pr-4">App rules</th>
              <th className="py-2 pr-4">Domain rules</th>
              <th className="py-2">Enabled</th>
            </tr>
          </thead>
          <tbody>
            {overview.split_templates.map((t) => (
              <tr key={t.id} className="border-b border-slate-900">
                <td className="py-2 pr-4">{t.name}</td>
                <td className="py-2 pr-4">{t.template_mode}</td>
                <td className="py-2 pr-4">{t.app_rules_count}</td>
                <td className="py-2 pr-4">{t.domain_rules_count}</td>
                <td className="py-2">{t.enabled ? "Yes" : "No"}</td>
              </tr>
            ))}
          </tbody>
        </table>
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
