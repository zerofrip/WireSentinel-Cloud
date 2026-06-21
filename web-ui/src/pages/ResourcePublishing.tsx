import { useEffect, useState } from "react";
import { fetchPublishedResources } from "../api";

export function ResourcePublishingPage() {
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
      <h1 className="text-2xl font-semibold">Resource Publishing</h1>
      <div className="rounded-lg border border-slate-800 overflow-hidden">
        <table className="w-full text-sm">
          <thead className="bg-slate-900 text-slate-400">
            <tr>
              <th className="text-left p-3">Name</th>
              <th className="text-left p-3">Endpoint</th>
              <th className="text-left p-3">Type</th>
              <th className="text-left p-3">Status</th>
            </tr>
          </thead>
          <tbody>
            {resources.map((r) => (
              <tr key={String(r.id)} className="border-t border-slate-800">
                <td className="p-3">{String(r.name)}</td>
                <td className="p-3 font-mono text-xs">
                  {String(r.host)}:{String(r.port)}
                </td>
                <td className="p-3">{String(r.resource_type)}</td>
                <td className="p-3">{r.published ? "Published" : "Draft"}</td>
              </tr>
            ))}
          </tbody>
        </table>
        {resources.length === 0 && (
          <p className="p-4 text-sm text-slate-500">No resources published yet.</p>
        )}
      </div>
    </div>
  );
}
