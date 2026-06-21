import { useEffect, useState } from "react";
import { fetchCloudAiReports, type AiReportRecord } from "../api";

export function AiCompliancePage() {
  const [reports, setReports] = useState<AiReportRecord[]>([]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    fetchCloudAiReports()
      .then((rows) => setReports(rows.filter((r) => r.report_type === "compliance")))
      .catch((e: Error) => setError(e.message));
  }, []);

  if (error) return <p className="text-red-400">{error}</p>;
  if (!reports.length) return <p>No AI compliance reports available yet.</p>;

  const avgCompliance =
    reports.reduce((sum, r) => sum + r.compliance_pct, 0) / reports.length;

  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-semibold">AI Compliance</h1>
      <div className="rounded-lg border border-slate-800 bg-slate-900 p-4 w-fit">
        <div className="text-sm text-slate-400">Average compliance</div>
        <div className="text-3xl font-bold mt-2">{Math.round(avgCompliance)}%</div>
      </div>
      <div className="overflow-x-auto">
        <table className="w-full text-sm">
          <thead>
            <tr className="text-left text-slate-400 border-b border-slate-800">
              <th className="py-2 pr-4">Title</th>
              <th className="py-2 pr-4">Status</th>
              <th className="py-2 pr-4">Compliance</th>
              <th className="py-2">Period</th>
            </tr>
          </thead>
          <tbody>
            {reports.map((r) => (
              <tr key={r.id} className="border-b border-slate-900">
                <td className="py-2 pr-4">{r.title}</td>
                <td className="py-2 pr-4">{r.status}</td>
                <td className="py-2 pr-4">{Math.round(r.compliance_pct)}%</td>
                <td className="py-2">
                  {r.period_start ?? "—"} → {r.period_end ?? "—"}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
