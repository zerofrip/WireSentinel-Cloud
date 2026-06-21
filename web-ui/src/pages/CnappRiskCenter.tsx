import { useEffect, useState } from "react";
import { fetchCloudCnappPosture, type CnappPostureRecord } from "../api";

export function CnappRiskCenterPage() {
  const [posture, setPosture] = useState<CnappPostureRecord[]>([]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    fetchCloudCnappPosture()
      .then(setPosture)
      .catch((e: Error) => setError(e.message));
  }, []);

  if (error) return <p className="text-red-400">{error}</p>;
  if (!posture.length) return <p>Loading CNAPP risk center…</p>;

  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-semibold">CNAPP Risk Center</h1>
      <div className="overflow-x-auto">
        <table className="w-full text-sm">
          <thead>
            <tr className="text-left text-slate-400 border-b border-slate-800">
              <th className="py-2 pr-4">Provider</th>
              <th className="py-2 pr-4">Account</th>
              <th className="py-2 pr-4">Score</th>
              <th className="py-2 pr-4">Risk</th>
              <th className="py-2">Findings</th>
            </tr>
          </thead>
          <tbody>
            {posture.map((p) => (
              <tr key={p.id} className="border-b border-slate-900">
                <td className="py-2 pr-4">{p.cloud_provider}</td>
                <td className="py-2 pr-4">{p.account_id ?? "—"}</td>
                <td className="py-2 pr-4">{Math.round(p.posture_score)}</td>
                <td className="py-2 pr-4">{p.risk_level}</td>
                <td className="py-2">{p.findings_count}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
