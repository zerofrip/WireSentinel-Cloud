import { useEffect, useState } from "react";
import { fetchPublishedResources } from "../api";

export function AccessPoliciesPage() {
  const [resources, setResources] = useState<Array<Record<string, unknown>>>([]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    fetchPublishedResources()
      .then(setResources)
      .catch((e: Error) => setError(e.message));
  }, []);

  if (error) return <p className="text-red-400">{error}</p>;

  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-semibold">Access Policies</h1>
      <p className="text-sm text-slate-400">
        Published resources and their attached ZTNA access policies.
      </p>
      {resources.length === 0 ? (
        <p className="text-slate-500">No published resources.</p>
      ) : (
        <table className="w-full text-sm">
          <thead className="text-slate-400">
            <tr>
              <th className="text-left p-2">Name</th>
              <th className="text-left p-2">Host</th>
              <th className="text-left p-2">Policy</th>
              <th className="text-left p-2">Published</th>
            </tr>
          </thead>
          <tbody>
            {resources.map((r) => (
              <tr key={String(r.id)} className="border-t border-slate-800">
                <td className="p-2">{String(r.name)}</td>
                <td className="p-2 font-mono text-xs">{String(r.host)}</td>
                <td className="p-2">{String(r.access_policy_id ?? "—")}</td>
                <td className="p-2">{r.published ? "Yes" : "No"}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </div>
  );
}
