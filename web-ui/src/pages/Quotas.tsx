import { useEffect, useState } from "react";
import { fetchQuotas } from "../api";

export function QuotasPage() {
  const [quotas, setQuotas] = useState<Array<Record<string, unknown>>>([]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    fetchQuotas()
      .then(setQuotas)
      .catch((e: Error) => setError(e.message));
  }, []);

  if (error) return <p className="text-red-400">{error}</p>;

  return (
    <div>
      <h1 className="text-2xl font-semibold mb-6">Quotas</h1>
      <div className="grid md:grid-cols-2 gap-4">
        {quotas.map((q) => {
          const usage = Number(q.current_usage);
          const hard = Number(q.hard_limit);
          const pct = hard > 0 ? Math.min(100, (usage / hard) * 100) : 0;
          return (
            <div key={String(q.resource)} className="rounded border border-slate-800 p-4">
              <div className="font-semibold capitalize">{String(q.resource)}</div>
              <div className="text-sm text-slate-400 mt-1">
                {usage} / {hard} (soft {String(q.soft_limit)})
              </div>
              <div className="mt-3 h-2 bg-slate-800 rounded">
                <div className="h-2 bg-violet-600 rounded" style={{ width: `${pct}%` }} />
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}
