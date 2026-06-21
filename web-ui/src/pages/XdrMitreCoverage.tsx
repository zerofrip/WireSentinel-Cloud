import { useEffect, useState } from "react";
import { fetchCloudXdrMitreCoverage, type XdrMitreCoverageRecord } from "../api";

export function XdrMitreCoveragePage() {
  const [coverage, setCoverage] = useState<XdrMitreCoverageRecord[]>([]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    fetchCloudXdrMitreCoverage()
      .then(setCoverage)
      .catch((e: Error) => setError(e.message));
  }, []);

  if (error) return <p className="text-red-400">{error}</p>;
  if (!coverage.length) return <p>No MITRE ATT&CK coverage data yet.</p>;

  const avgCoverage =
    coverage.reduce((sum, c) => sum + c.coverage_pct, 0) / coverage.length;

  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-semibold">XDR MITRE Coverage</h1>
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        <Stat label="Techniques tracked" value={coverage.length} />
        <Stat label="Avg coverage" value={Math.round(avgCoverage)} suffix="%" />
        <Stat
          label="Total detections"
          value={coverage.reduce((sum, c) => sum + c.detection_count, 0)}
        />
      </div>
      <div className="overflow-x-auto rounded-lg border border-slate-800">
        <table className="w-full text-sm">
          <thead className="bg-slate-900 text-slate-400">
            <tr>
              <th className="px-4 py-2 text-left">Tactic</th>
              <th className="px-4 py-2 text-left">Technique</th>
              <th className="px-4 py-2 text-left">Detections</th>
              <th className="px-4 py-2 text-left">Coverage</th>
            </tr>
          </thead>
          <tbody>
            {coverage.map((c) => (
              <tr key={c.id} className="border-t border-slate-800">
                <td className="px-4 py-2">{c.tactic}</td>
                <td className="px-4 py-2">{c.technique_id} — {c.technique_name}</td>
                <td className="px-4 py-2">{c.detection_count}</td>
                <td className="px-4 py-2">{Math.round(c.coverage_pct)}%</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}

function Stat({ label, value, suffix = "" }: { label: string; value: number; suffix?: string }) {
  return (
    <div className="rounded-lg border border-slate-800 bg-slate-900 p-4">
      <div className="text-sm text-slate-400">{label}</div>
      <div className="text-3xl font-bold mt-2">{value}{suffix}</div>
    </div>
  );
}
