import { useEffect, useState } from "react";
import { fetchUsageAggregates } from "../api";

export function UsagePage() {
  const [rows, setRows] = useState<Array<Record<string, unknown>>>([]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    fetchUsageAggregates()
      .then(setRows)
      .catch((e: Error) => setError(e.message));
  }, []);

  if (error) return <p className="text-red-400">{error}</p>;

  return (
    <div>
      <h1 className="text-2xl font-semibold mb-6">Usage</h1>
      <table className="w-full text-sm">
        <thead>
          <tr className="text-left text-slate-400 border-b border-slate-800">
            <th className="py-2">Metric</th>
            <th>Period</th>
            <th>Total</th>
            <th>Peak</th>
          </tr>
        </thead>
        <tbody>
          {rows.map((r) => (
            <tr key={`${r.metric}-${r.period}`} className="border-b border-slate-900">
              <td className="py-2">{String(r.metric)}</td>
              <td>{String(r.period)}</td>
              <td>{String(r.total)}</td>
              <td>{String(r.peak)}</td>
            </tr>
          ))}
        </tbody>
      </table>
      {rows.length === 0 && <p className="text-slate-400 mt-4">No usage recorded yet.</p>}
    </div>
  );
}
