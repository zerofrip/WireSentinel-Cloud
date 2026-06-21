import { useEffect, useState } from "react";
import { fetchCloudXdrIncidents, type XdrIncidentRecord } from "../api";

export function XdrIncidentCenterPage() {
  const [incidents, setIncidents] = useState<XdrIncidentRecord[]>([]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    fetchCloudXdrIncidents()
      .then(setIncidents)
      .catch((e: Error) => setError(e.message));
  }, []);

  if (error) return <p className="text-red-400">{error}</p>;
  if (!incidents.length) return <p>No XDR incidents recorded yet.</p>;

  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-semibold">XDR Incident Center</h1>
      <div className="overflow-x-auto rounded-lg border border-slate-800">
        <table className="w-full text-sm">
          <thead className="bg-slate-900 text-slate-400">
            <tr>
              <th className="px-4 py-2 text-left">Title</th>
              <th className="px-4 py-2 text-left">Status</th>
              <th className="px-4 py-2 text-left">Severity</th>
              <th className="px-4 py-2 text-left">Detections</th>
              <th className="px-4 py-2 text-left">Opened</th>
            </tr>
          </thead>
          <tbody>
            {incidents.map((inc) => (
              <tr key={inc.id} className="border-t border-slate-800">
                <td className="px-4 py-2">{inc.title}</td>
                <td className="px-4 py-2">{inc.status}</td>
                <td className="px-4 py-2">{inc.severity}</td>
                <td className="px-4 py-2">{inc.detection_count}</td>
                <td className="px-4 py-2">{inc.opened_at}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
