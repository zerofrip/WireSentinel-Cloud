import { useEffect, useState } from "react";
import type { ZtnaFleetOverview } from "../api";

export function IdentityPage() {
  const [providers, setProviders] = useState<Array<Record<string, unknown>>>([]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    import("../api")
      .then((api) => api.fetchIdentityProviders())
      .then(setProviders)
      .catch((e: Error) => setError(e.message));
  }, []);

  if (error) return <p className="text-red-400">{error}</p>;

  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-semibold">Identity Providers</h1>
      {providers.length === 0 ? (
        <p className="text-slate-400">No identity providers configured.</p>
      ) : (
        <ul className="space-y-2">
          {providers.map((p) => (
            <li key={String(p.id)} className="rounded border border-slate-800 bg-slate-900 p-3 text-sm">
              <strong>{String(p.name)}</strong> · {String(p.provider_kind)}
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}

export function ZtnaFleetWidget({ overview }: { overview: ZtnaFleetOverview | null }) {
  if (!overview) return null;
  return (
    <div className="rounded-lg border border-slate-800 bg-slate-900 p-4">
      <h3 className="text-sm text-slate-400 mb-2">ZTNA fleet</h3>
      <p className="text-sm">
        Devices reporting: <strong>{overview.reporting_devices}</strong> · Avg trust:{" "}
        <strong>{overview.avg_trust_score.toFixed(1)}</strong> · Denials:{" "}
        <strong>{overview.deny_count}</strong>
      </p>
    </div>
  );
}
