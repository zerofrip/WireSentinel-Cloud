import { useEffect, useState } from "react";
import { fetchCloudTcpTermination, type WiresockTcpTerminationRecord } from "../api";

export function WiresockTcpTerminationPage() {
  const [rules, setRules] = useState<WiresockTcpTerminationRecord[]>([]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    fetchCloudTcpTermination()
      .then(setRules)
      .catch((e: Error) => setError(e.message));
  }, []);

  if (error) return <p className="text-red-400">{error}</p>;
  if (!rules.length) return <p>Loading TCP termination rules…</p>;

  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-semibold">TCP Termination</h1>
      <div className="overflow-x-auto">
        <table className="w-full text-sm">
          <thead>
            <tr className="text-left text-slate-400 border-b border-slate-800">
              <th className="py-2 pr-4">Rule</th>
              <th className="py-2 pr-4">Mode</th>
              <th className="py-2 pr-4">Process</th>
              <th className="py-2 pr-4">Profile</th>
              <th className="py-2">Enabled</th>
            </tr>
          </thead>
          <tbody>
            {rules.map((r) => (
              <tr key={r.id} className="border-b border-slate-900">
                <td className="py-2 pr-4">{r.rule_name}</td>
                <td className="py-2 pr-4">{r.mode}</td>
                <td className="py-2 pr-4">{r.process_name ?? "—"}</td>
                <td className="py-2 pr-4">{r.profile_id ?? "—"}</td>
                <td className="py-2">{r.enabled ? "Yes" : "No"}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
