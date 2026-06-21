import { useEffect, useState } from "react";
import {
  fetchBillingPlans,
  fetchBillingSubscription,
  fetchControllers,
  fetchHealth,
  fetchMetrics,
  fetchQuotas,
  fetchRegions,
  fetchTenants,
  fetchUsageAggregates,
  fetchCloudZtna,
  fetchCloudSse,
} from "../api";
import { ZtnaFleetWidget } from "./Identity";

export function DashboardPage() {
  const [metrics, setMetrics] = useState<Record<string, number> | null>(null);
  const [health, setHealth] = useState<string | null>(null);
  const [mrrCents, setMrrCents] = useState(0);
  const [activeTenants, setActiveTenants] = useState(0);
  const [controllerCount, setControllerCount] = useState(0);
  const [bandwidthBytes, setBandwidthBytes] = useState(0);
  const [quotaOk, setQuotaOk] = useState(true);
  const [regionsHealthy, setRegionsHealthy] = useState(0);
  const [regionsTotal, setRegionsTotal] = useState(0);
  const [ztnaOverview, setZtnaOverview] = useState<Awaited<ReturnType<typeof fetchCloudZtna>> | null>(null);
  const [sseOverview, setSseOverview] = useState<Awaited<ReturnType<typeof fetchCloudSse>> | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    Promise.all([
      fetchMetrics(),
      fetchHealth(),
      fetchTenants(),
      fetchControllers(),
      fetchBillingSubscription(),
      fetchBillingPlans(),
      fetchUsageAggregates(),
      fetchQuotas(),
      fetchRegions(),
      fetchCloudZtna().catch(() => null),
      fetchCloudSse().catch(() => null),
    ])
      .then(([m, h, tenants, controllers, sub, plans, usage, quotas, regions, ztna, sse]) => {
        setMetrics(m);
        setHealth(h.service);
        setActiveTenants(tenants.length);
        setControllerCount(controllers.length);

        const planId = sub?.plan ? String(sub.plan) : "free";
        const plan = plans.find((p) => p.id === planId);
        setMrrCents(Number(plan?.price_cents ?? 0));

        const bw = usage.find((u) => u.metric === "bandwidth_bytes");
        setBandwidthBytes(Number(bw?.total ?? 0));

        const exceeded = quotas.some((q) => Number(q.current_usage) >= Number(q.hard_limit));
        setQuotaOk(!exceeded);

        setRegionsTotal(regions.length);
        setRegionsHealthy(regions.filter((r) => r.healthy).length);
        setZtnaOverview(ztna);
        setSseOverview(sse);
      })
      .catch((e: Error) => setError(e.message));
  }, []);

  if (error) return <p className="text-red-400">{error}</p>;
  if (!metrics) return <p>Loading…</p>;

  const cards: [string, string][] = [
    ["MRR", `$${(mrrCents / 100).toFixed(2)}`],
    ["Active tenants", String(activeTenants)],
    ["Controllers", String(controllerCount)],
    ["Bandwidth (bytes)", bandwidthBytes.toLocaleString()],
    ["Quota status", quotaOk ? "Within limits" : "Exceeded"],
    ["Region health", `${regionsHealthy}/${regionsTotal} healthy`],
    ["Organizations", String(metrics.organizations ?? 0)],
    ["Teams", String(metrics.teams ?? 0)],
    ["Open sync conflicts", String(metrics.open_sync_conflicts ?? 0)],
  ];
  if (sseOverview) {
    cards.push(["SSE threats", String(sseOverview.threat_count)]);
    cards.push(["SSE blocked", String(sseOverview.swg_blocked)]);
  }

  return (
    <div>
      <h1 className="text-2xl font-semibold mb-2">Dashboard</h1>
      <p className="text-sm text-slate-400 mb-6">Service: {health ?? "—"}</p>
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        {cards.map(([label, value]) => (
          <div key={label} className="rounded-lg border border-slate-800 bg-slate-900 p-4">
            <div className="text-sm text-slate-400">{label}</div>
            <div className="text-2xl font-bold mt-2">{value}</div>
          </div>
        ))}
      </div>
      <div className="mt-6">
        <ZtnaFleetWidget overview={ztnaOverview} />
      </div>
    </div>
  );
}
