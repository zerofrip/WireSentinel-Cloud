import { useEffect, useState } from "react";
import { fetchCloudCnappCompliance, type CnappComplianceRecord } from "../api";

export function CnappComplianceCenterPage() {
  const [compliance, setCompliance] = useState<CnappComplianceRecord[]>([]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    fetchCloudCnappCompliance()
      .then(setCompliance)
      .catch((e: Error) => setError(e.message));
  }, []);

  if (error) return <p className="text-red-400">{error}</p>;
  if (!compliance.length) return <p>Loading CNAPP compliance center…</p>;

  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-semibold">CNAPP Compliance Center</h1>
      <div className="overflow-x-auto">
        <table className="w-full text-sm">
          <thead>
            <tr className="text-left text-slate-400 border-b border-slate-800">
              <th className="py-2 pr-4">Framework</th>
              <th className="py-2 pr-4">Control</th>
              <th className="py-2 pr-4">Status</th>
              <th className="py-2">Compliance</th>
            </tr>
          </thead>
          <tbody>
            {compliance.map((c) => (
              <tr key={c.id} className="border-b border-slate-900">
                <td className="py-2 pr-4">{c.framework}</td>
                <td className="py-2 pr-4">{c.control_name}</td>
                <td className="py-2 pr-4">{c.status}</td>
                <td className="py-2">{Math.round(c.compliance_pct)}%</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
