import { useEffect, useState } from "react";
import { fetchBillingInvoices, fetchBillingPlans, fetchBillingSubscription } from "../api";

export function BillingPage() {
  const [plans, setPlans] = useState<Array<Record<string, unknown>>>([]);
  const [sub, setSub] = useState<Record<string, unknown> | null>(null);
  const [invoices, setInvoices] = useState<Array<Record<string, unknown>>>([]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    Promise.all([fetchBillingPlans(), fetchBillingSubscription(), fetchBillingInvoices()])
      .then(([p, s, i]) => {
        setPlans(p);
        setSub(s);
        setInvoices(i);
      })
      .catch((e: Error) => setError(e.message));
  }, []);

  if (error) return <p className="text-red-400">{error}</p>;

  return (
    <div>
      <h1 className="text-2xl font-semibold mb-6">Billing</h1>
      {sub && (
        <div className="rounded border border-slate-800 p-4 mb-6">
          Active plan: <strong>{String(sub.plan)}</strong> · Status: {String(sub.status)}
        </div>
      )}
      <h2 className="text-lg mb-3">Plans</h2>
      <div className="grid md:grid-cols-3 gap-4 mb-8">
        {plans.map((p) => (
          <div key={String(p.id)} className="rounded border border-slate-800 p-4">
            <div className="font-semibold">{String(p.name)}</div>
            <div className="text-sm text-slate-400">${(Number(p.price_cents) / 100).toFixed(2)}/mo</div>
          </div>
        ))}
      </div>
      <h2 className="text-lg mb-3">Invoices</h2>
      <ul className="space-y-2">
        {invoices.map((inv) => (
          <li key={String(inv.id)} className="rounded border border-slate-800 p-3 text-sm">
            {String(inv.status)} · ${(Number(inv.amount_cents) / 100).toFixed(2)} · {String(inv.created_at)}
          </li>
        ))}
        {invoices.length === 0 && <li className="text-slate-400 text-sm">No invoices</li>}
      </ul>
    </div>
  );
}
