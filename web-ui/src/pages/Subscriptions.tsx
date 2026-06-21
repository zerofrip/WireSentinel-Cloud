import { useEffect, useState } from "react";
import { fetchPlans, fetchSubscriptions } from "../api";

export function SubscriptionsPage() {
  const [subs, setSubs] = useState<Array<Record<string, unknown>>>([]);
  const [plans, setPlans] = useState<Array<Record<string, unknown>>>([]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    Promise.all([fetchSubscriptions(), fetchPlans()])
      .then(([s, p]) => {
        setSubs(s);
        setPlans(p);
      })
      .catch((e: Error) => setError(e.message));
  }, []);

  if (error) return <p className="text-red-400">{error}</p>;

  return (
    <div>
      <h1 className="text-2xl font-semibold mb-6">Subscriptions</h1>
      <h2 className="text-lg mb-3">Current</h2>
      <ul className="space-y-2 mb-8">
        {subs.map((s) => (
          <li key={String(s.id)} className="rounded border border-slate-800 p-3">
            Plan: {String(s.plan)} · Status: {String(s.status)} · Seats: {String(s.seats)}
          </li>
        ))}
        {subs.length === 0 && <li className="text-slate-400 text-sm">No subscriptions</li>}
      </ul>
      <h2 className="text-lg mb-3">Available plans</h2>
      <div className="grid md:grid-cols-3 gap-4">
        {plans.map((p) => (
          <div key={String(p.id)} className="rounded border border-slate-800 p-4">
            <div className="font-semibold">{String(p.name)}</div>
            <div className="text-xs text-slate-400 mt-2">
              {JSON.stringify(p.limits)}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
