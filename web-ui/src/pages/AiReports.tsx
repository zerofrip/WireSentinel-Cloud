import { useEffect, useState } from "react";
import { fetchCloudAiReports, type AiReportRecord } from "../api";

export function AiReportsPage() {
  const [reports, setReports] = useState<AiReportRecord[]>([]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    fetchCloudAiReports()
      .then(setReports)
      .catch((e: Error) => setError(e.message));
  }, []);

  if (error) return <p className="text-red-400">{error}</p>;
  if (!reports.length) return <p>No AI security reports generated yet.</p>;

  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-semibold">AI Security Reports</h1>
      <div className="overflow-x-auto">
        <table className="w-full text-sm">
          <thead>
            <tr className="text-left text-slate-400 border-b border-slate-800">
              <th className="py-2 pr-4">Title</th>
              <th className="py-2 pr-4">Type</th>
              <th className="py-2 pr-4">Status</th>
              <th className="py-2 pr-4">Compliance</th>
              <th className="py-2">Generated</th>
            </tr>
          </thead>
          <tbody>
            {reports.map((r) => (
              <tr key={r.id} className="border-b border-slate-900">
                <td className="py-2 pr-4">{r.title}</td>
                <td className="py-2 pr-4">{r.report_type}</td>
                <td className="py-2 pr-4">{r.status}</td>
                <td className="py-2 pr-4">{Math.round(r.compliance_pct)}%</td>
                <td className="py-2">{r.generated_at}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
