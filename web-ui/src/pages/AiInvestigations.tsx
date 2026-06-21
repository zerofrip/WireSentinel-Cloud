import { useEffect, useState } from "react";
import { fetchCloudAiInvestigations, type AiInvestigationRecord } from "../api";

export function AiInvestigationsPage() {
  const [investigations, setInvestigations] = useState<AiInvestigationRecord[]>([]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    fetchCloudAiInvestigations()
      .then(setInvestigations)
      .catch((e: Error) => setError(e.message));
  }, []);

  if (error) return <p className="text-red-400">{error}</p>;
  if (!investigations.length) return <p>No AI investigations recorded yet.</p>;

  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-semibold">AI Investigations</h1>
      <div className="overflow-x-auto">
        <table className="w-full text-sm">
          <thead>
            <tr className="text-left text-slate-400 border-b border-slate-800">
              <th className="py-2 pr-4">Title</th>
              <th className="py-2 pr-4">Category</th>
              <th className="py-2 pr-4">Severity</th>
              <th className="py-2 pr-4">Status</th>
              <th className="py-2">Findings</th>
            </tr>
          </thead>
          <tbody>
            {investigations.map((inv) => (
              <tr key={inv.id} className="border-b border-slate-900">
                <td className="py-2 pr-4">{inv.title}</td>
                <td className="py-2 pr-4">{inv.category}</td>
                <td className="py-2 pr-4">{inv.severity}</td>
                <td className="py-2 pr-4">{inv.status}</td>
                <td className="py-2">{inv.finding_count}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
