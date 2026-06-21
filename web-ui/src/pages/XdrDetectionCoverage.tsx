import { useEffect, useState } from "react";
import { fetchCloudXdrDetections, type XdrDetectionRecord } from "../api";

export function XdrDetectionCoveragePage() {
  const [detections, setDetections] = useState<XdrDetectionRecord[]>([]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    fetchCloudXdrDetections()
      .then(setDetections)
      .catch((e: Error) => setError(e.message));
  }, []);

  if (error) return <p className="text-red-400">{error}</p>;
  if (!detections.length) return <p>No XDR detections recorded yet.</p>;

  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-semibold">XDR Detection Coverage</h1>
      <div className="overflow-x-auto rounded-lg border border-slate-800">
        <table className="w-full text-sm">
          <thead className="bg-slate-900 text-slate-400">
            <tr>
              <th className="px-4 py-2 text-left">Rule</th>
              <th className="px-4 py-2 text-left">Kind</th>
              <th className="px-4 py-2 text-left">Severity</th>
              <th className="px-4 py-2 text-left">MITRE</th>
              <th className="px-4 py-2 text-left">Matched</th>
            </tr>
          </thead>
          <tbody>
            {detections.map((d) => (
              <tr key={d.id} className="border-t border-slate-800">
                <td className="px-4 py-2">{d.rule_name}</td>
                <td className="px-4 py-2">{d.rule_kind}</td>
                <td className="px-4 py-2">{d.severity}</td>
                <td className="px-4 py-2">{d.mitre_technique_id ?? "—"}</td>
                <td className="px-4 py-2">{d.matched_at}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
