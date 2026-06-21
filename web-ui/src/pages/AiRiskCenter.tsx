import { useEffect, useState } from "react";
import { fetchCloudAiRisk, type AiRiskRecord } from "../api";

export function AiRiskCenterPage() {
  const [risks, setRisks] = useState<AiRiskRecord[]>([]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    fetchCloudAiRisk()
      .then(setRisks)
      .catch((e: Error) => setError(e.message));
  }, []);

  if (error) return <p className="text-red-400">{error}</p>;
  if (!risks.length) return <p>No AI risk assessments recorded yet.</p>;

  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-semibold">AI Risk Center</h1>
      <div className="overflow-x-auto">
        <table className="w-full text-sm">
          <thead>
            <tr className="text-left text-slate-400 border-b border-slate-800">
              <th className="py-2 pr-4">Category</th>
              <th className="py-2 pr-4">Model</th>
              <th className="py-2 pr-4">Score</th>
              <th className="py-2 pr-4">Severity</th>
              <th className="py-2">Status</th>
            </tr>
          </thead>
          <tbody>
            {risks.map((r) => (
              <tr key={r.id} className="border-b border-slate-900">
                <td className="py-2 pr-4">{r.risk_category}</td>
                <td className="py-2 pr-4">{r.model_name ?? "—"}</td>
                <td className="py-2 pr-4">{Math.round(r.risk_score)}</td>
                <td className="py-2 pr-4">{r.severity}</td>
                <td className="py-2">{r.status}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
