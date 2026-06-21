import { useEffect, useState } from "react";
import { fetchCloudCnapp, type CnappAttackPathRecord } from "../api";

export function CnappAttackPathsPage() {
  const [attackPaths, setAttackPaths] = useState<CnappAttackPathRecord[]>([]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    fetchCloudCnapp()
      .then((overview) => setAttackPaths(overview.attack_paths))
      .catch((e: Error) => setError(e.message));
  }, []);

  if (error) return <p className="text-red-400">{error}</p>;
  if (!attackPaths.length) return <p>No attack paths detected.</p>;

  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-semibold">CNAPP Attack Paths</h1>
      <div className="overflow-x-auto">
        <table className="w-full text-sm">
          <thead>
            <tr className="text-left text-slate-400 border-b border-slate-800">
              <th className="py-2 pr-4">Name</th>
              <th className="py-2 pr-4">Severity</th>
              <th className="py-2 pr-4">Entry</th>
              <th className="py-2 pr-4">Target</th>
              <th className="py-2">Status</th>
            </tr>
          </thead>
          <tbody>
            {attackPaths.map((p) => (
              <tr key={p.id} className="border-b border-slate-900">
                <td className="py-2 pr-4">{p.name}</td>
                <td className="py-2 pr-4">{p.severity}</td>
                <td className="py-2 pr-4">{p.entry_point ?? "—"}</td>
                <td className="py-2 pr-4">{p.target_asset ?? "—"}</td>
                <td className="py-2">{p.status}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
