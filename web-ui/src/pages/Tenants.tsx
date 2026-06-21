import { useEffect, useState } from "react";
import { fetchTenants } from "../api";

export function TenantsPage() {
  const [tenants, setTenants] = useState<Array<Record<string, unknown>>>([]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    fetchTenants()
      .then(setTenants)
      .catch((e: Error) => setError(e.message));
  }, []);

  if (error) return <p className="text-red-400">{error}</p>;

  return (
    <div>
      <h1 className="text-2xl font-semibold mb-6">Tenants</h1>
      <div className="rounded border border-slate-800 overflow-hidden">
        <table className="w-full text-sm">
          <thead className="bg-slate-900 text-slate-400">
            <tr>
              <th className="text-left p-3">Name</th>
              <th className="text-left p-3">Slug</th>
              <th className="text-left p-3">Status</th>
            </tr>
          </thead>
          <tbody>
            {tenants.map((t) => (
              <tr key={String(t.id)} className="border-t border-slate-800">
                <td className="p-3">{String(t.name)}</td>
                <td className="p-3">{String(t.slug)}</td>
                <td className="p-3">{String(t.status)}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
