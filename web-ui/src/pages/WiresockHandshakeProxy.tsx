import { useEffect, useState } from "react";
import { fetchCloudHandshakeProxy, type WiresockHandshakeProxyRecord } from "../api";

export function WiresockHandshakeProxyPage() {
  const [proxies, setProxies] = useState<WiresockHandshakeProxyRecord[]>([]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    fetchCloudHandshakeProxy()
      .then(setProxies)
      .catch((e: Error) => setError(e.message));
  }, []);

  if (error) return <p className="text-red-400">{error}</p>;
  if (!proxies.length) return <p>Loading handshake proxy configs…</p>;

  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-semibold">Handshake Proxy</h1>
      <div className="overflow-x-auto">
        <table className="w-full text-sm">
          <thead>
            <tr className="text-left text-slate-400 border-b border-slate-800">
              <th className="py-2 pr-4">Name</th>
              <th className="py-2 pr-4">Type</th>
              <th className="py-2 pr-4">Endpoint</th>
              <th className="py-2">Enabled</th>
            </tr>
          </thead>
          <tbody>
            {proxies.map((p) => (
              <tr key={p.id} className="border-b border-slate-900">
                <td className="py-2 pr-4">{p.name}</td>
                <td className="py-2 pr-4">{p.proxy_type}</td>
                <td className="py-2 pr-4">{p.endpoint ?? "—"}</td>
                <td className="py-2">{p.enabled ? "Yes" : "No"}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
